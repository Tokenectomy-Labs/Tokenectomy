use crate::redact::redact_secrets;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Default)]
pub struct ProxySanitizeStats {
    pub raw_chars: usize,
    pub sanitized_chars: usize,
    pub secrets_redacted: usize,
}

#[derive(Clone, Debug)]
pub struct CachedCompletion {
    pub status: u16,
    pub reason: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub timestamp: Instant,
    pub ttl: Duration,
    pub estimated_tokens: u64,
}

pub type SharedResponseCache = Arc<RwLock<HashMap<String, CachedCompletion>>>;

/// Configuration options for the Tokenectomy AI Gateway.
#[derive(Clone, Debug)]
pub struct ProxyConfig {
    pub bind_addr: String,
    pub upstream_url: String,
    pub allow_remote: bool,
    pub auth_token: Option<String>,
    pub max_hourly_tokens: Option<u64>,
    pub max_retries: usize,
    pub auto_retry_429: bool,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:8080".to_string(),
            upstream_url: "auto".to_string(),
            allow_remote: false,
            auth_token: None,
            max_hourly_tokens: None,
            max_retries: 3,
            auto_retry_429: true,
        }
    }
}

/// Real-time thread-safe metrics collector for FinOps and token economics monitoring.
#[derive(Debug)]
pub struct ProxyMetrics {
    pub start_time: Instant,
    pub total_requests: AtomicU64,
    pub total_raw_chars: AtomicU64,
    pub total_sanitized_chars: AtomicU64,
    pub total_secrets_redacted: AtomicU64,
    pub total_cache_hits: AtomicU64,
    pub total_cache_tokens_saved: AtomicU64,
    pub total_rate_limits_mitigated: AtomicU64,
    pub total_circuit_breaker_trips: AtomicU64,
    pub last_latency_ms: AtomicU64,
    pub hourly_token_history: Mutex<VecDeque<(Instant, u64)>>,
}

impl Default for ProxyMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ProxyMetrics {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            total_requests: AtomicU64::new(0),
            total_raw_chars: AtomicU64::new(0),
            total_sanitized_chars: AtomicU64::new(0),
            total_secrets_redacted: AtomicU64::new(0),
            total_cache_hits: AtomicU64::new(0),
            total_cache_tokens_saved: AtomicU64::new(0),
            total_rate_limits_mitigated: AtomicU64::new(0),
            total_circuit_breaker_trips: AtomicU64::new(0),
            last_latency_ms: AtomicU64::new(0),
            hourly_token_history: Mutex::new(VecDeque::new()),
        }
    }

    pub fn record(&self, raw: usize, sanitized: usize, secrets: usize) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_raw_chars.fetch_add(raw as u64, Ordering::Relaxed);
        self.total_sanitized_chars.fetch_add(sanitized as u64, Ordering::Relaxed);
        self.total_secrets_redacted.fetch_add(secrets as u64, Ordering::Relaxed);
    }

    pub fn record_request(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_hit(&self, saved_tokens: u64) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_cache_hits.fetch_add(1, Ordering::Relaxed);
        self.total_cache_tokens_saved.fetch_add(saved_tokens, Ordering::Relaxed);
    }

    pub fn record_rate_limit_mitigated(&self) {
        self.total_rate_limits_mitigated.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_circuit_breaker_trip(&self) {
        self.total_circuit_breaker_trips.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_latency(&self, ms: u64) {
        self.last_latency_ms.store(ms, Ordering::Relaxed);
    }

    pub fn check_and_record_hourly_tokens(&self, new_tokens: u64, max_hourly: Option<u64>) -> bool {
        let now = Instant::now();
        let one_hour = Duration::from_secs(3600);
        if let Ok(mut history) = self.hourly_token_history.lock() {
            while let Some((ts, _)) = history.front() {
                if now.duration_since(*ts) > one_hour {
                    history.pop_front();
                } else {
                    break;
                }
            }

            let current_hourly: u64 = history.iter().fold(0u64, |acc, (_, t)| acc.saturating_add(*t));
            if let Some(limit) = max_hourly {
                if limit > 0 && current_hourly.saturating_add(new_tokens) > limit {
                    self.record_circuit_breaker_trip();
                    return false;
                }
            }

            history.push_back((now, new_tokens));
            true
        } else {
            true
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        let raw_chars = self.total_raw_chars.load(Ordering::Relaxed);
        let sanitized_chars = self.total_sanitized_chars.load(Ordering::Relaxed);
        let requests = self.total_requests.load(Ordering::Relaxed);
        let secrets = self.total_secrets_redacted.load(Ordering::Relaxed);
        let cache_hits = self.total_cache_hits.load(Ordering::Relaxed);
        let cache_tokens = self.total_cache_tokens_saved.load(Ordering::Relaxed);
        let rate_limits = self.total_rate_limits_mitigated.load(Ordering::Relaxed);
        let cb_trips = self.total_circuit_breaker_trips.load(Ordering::Relaxed);
        let last_latency = self.last_latency_ms.load(Ordering::Relaxed);
        let uptime = self.start_time.elapsed().as_secs();

        let current_hourly = if let Ok(mut history) = self.hourly_token_history.lock() {
            let now = Instant::now();
            let one_hour = Duration::from_secs(3600);
            while let Some((ts, _)) = history.front() {
                if now.duration_since(*ts) > one_hour {
                    history.pop_front();
                } else {
                    break;
                }
            }
            history.iter().map(|(_, t)| *t).sum()
        } else {
            0
        };

        let raw_tokens = raw_chars / 4;
        let sanitized_tokens = sanitized_chars / 4;
        let tokens_saved_by_surgery = raw_tokens.saturating_sub(sanitized_tokens);
        let total_tokens_saved = tokens_saved_by_surgery + cache_tokens;
        let saved_pct = if raw_chars > 0 {
            (raw_chars.saturating_sub(sanitized_chars) as f64 / raw_chars as f64) * 100.0
        } else {
            0.0
        };

        // Standard blended LLM input token pricing: ~$3.00 per 1M prompt tokens ($0.003/1K)
        let cost_saved_usd = (total_tokens_saved as f64 / 1_000_000.0) * 3.0;

        serde_json::json!({
            "status": "ok",
            "service": "tokenectomy-gateway",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime_seconds": uptime,
            "total_requests": requests,
            "raw_characters": raw_chars,
            "sanitized_characters": sanitized_chars,
            "estimated_raw_tokens": raw_tokens,
            "estimated_sanitized_tokens": sanitized_tokens,
            "estimated_tokens_saved": total_tokens_saved,
            "tokens_saved_by_surgery": tokens_saved_by_surgery,
            "cache_hits": cache_hits,
            "cache_tokens_saved": cache_tokens,
            "rate_limits_mitigated": rate_limits,
            "circuit_breaker_trips": cb_trips,
            "last_latency_ms": last_latency,
            "current_hourly_tokens": current_hourly,
            "reduction_percentage": (saved_pct * 10.0).round() / 10.0,
            "secrets_redacted": secrets,
            "estimated_cost_saved_usd": (cost_saved_usd * 1000.0).round() / 1000.0,
            "blended_rate_per_million": 3.00
        })
    }
}

/// Surgically cleans prompt payload: redacts secrets and strips framework dependency noise.
/// Point 13: Only sanitizes string content in role:"user" or type:"tool_result".
/// Never modifies assistant messages, thinking blocks, redacted_thinking, tool_use.id, cache_control, or image blocks.
pub fn sanitize_prompt_payload(payload: &Value) -> (Value, ProxySanitizeStats) {
    let mut stats = ProxySanitizeStats::default();
    let mut modified = payload.clone();

    fn clean_string(text: &str, stats: &mut ProxySanitizeStats) -> String {
        stats.raw_chars += text.len();
        let redacted = redact_secrets(text);
        if redacted != text {
            stats.secrets_redacted += 1;
        }
        let final_content = crate::extractor::prune_framework_noise(&redacted);
        stats.sanitized_chars += final_content.len();
        final_content
    }

    // 1. Process "messages" array (OpenAI, Anthropic, Ollama chat completions)
    if let Some(messages) = modified.get_mut("messages").and_then(|m| m.as_array_mut()) {
        for msg in messages {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");
            // Point 13: Strictly ignore assistant messages to preserve thinking signatures and model outputs
            if role == "assistant" {
                continue;
            }

            if let Some(content) = msg.get_mut("content") {
                match content {
                    Value::String(s) => {
                        *s = clean_string(s, &mut stats);
                    }
                    Value::Array(parts) => {
                        for part in parts {
                            match part {
                                Value::String(s) => {
                                    *s = clean_string(s, &mut stats);
                                }
                                Value::Object(_) => {
                                    let part_type = part.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                    // Point 13: Never touch thinking, redacted_thinking, tool_use, or image blocks
                                    if part_type == "thinking"
                                        || part_type == "redacted_thinking"
                                        || part_type == "image"
                                        || part_type == "tool_use"
                                    {
                                        continue;
                                    }

                                    // If tool_result block, sanitize content while preserving tool_use_id and cache_control
                                    if part_type == "tool_result" {
                                        if let Some(content_val) = part.get_mut("content") {
                                            match content_val {
                                                Value::String(s) => {
                                                    *s = clean_string(s, &mut stats);
                                                }
                                                Value::Array(nested_parts) => {
                                                    for np in nested_parts {
                                                        match np {
                                                            Value::String(s) => {
                                                                *s = clean_string(s, &mut stats);
                                                            }
                                                            Value::Object(_) => {
                                                                let np_type = np.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                                                if np_type == "image" || np_type == "thinking" {
                                                                    continue;
                                                                }
                                                                if let Some(t) = np.get_mut("text").and_then(|t| t.as_str()) {
                                                                    let cleaned = clean_string(t, &mut stats);
                                                                    np["text"] = Value::String(cleaned);
                                                                }
                                                            }
                                                            _ => {}
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        continue;
                                    }

                                    // Standard user text blocks
                                    if let Some(text_val) = part.get_mut("text").and_then(|t| t.as_str()) {
                                        let cleaned = clean_string(text_val, &mut stats);
                                        part["text"] = Value::String(cleaned);
                                    } else if let Some(content_val) = part.get_mut("content").and_then(|c| c.as_str()) {
                                        let cleaned = clean_string(content_val, &mut stats);
                                        part["content"] = Value::String(cleaned);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // 2. Process top-level "system" prompt (Anthropic Claude Messages API / OpenAI system)
    if let Some(system) = modified.get_mut("system") {
        match system {
            Value::String(s) => {
                *s = clean_string(s, &mut stats);
            }
            Value::Array(parts) => {
                for part in parts {
                    let part_type = part.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    if part_type == "image" || part_type == "thinking" {
                        continue;
                    }
                    if let Some(text_val) = part.get_mut("text").and_then(|t| t.as_str()) {
                        let cleaned = clean_string(text_val, &mut stats);
                        part["text"] = Value::String(cleaned);
                    }
                }
            }
            _ => {}
        }
    }

    // 3. Process top-level "prompt" (OpenAI Completions API, Ollama /api/generate)
    if let Some(prompt) = modified.get_mut("prompt") {
        if let Some(s) = prompt.as_str() {
            let cleaned = clean_string(s, &mut stats);
            *prompt = Value::String(cleaned);
        }
    }

    // 4. Process top-level "input" (OpenAI Embeddings / Transforms)
    if let Some(input) = modified.get_mut("input") {
        match input {
            Value::String(s) => {
                *s = clean_string(s, &mut stats);
            }
            Value::Array(items) => {
                for item in items {
                    if let Some(s) = item.as_str() {
                        let cleaned = clean_string(s, &mut stats);
                        *item = Value::String(cleaned);
                    }
                }
            }
            _ => {}
        }
    }

    (modified, stats)
}

pub const MAX_HEADER_SIZE: usize = 64 * 1024; // 64 KB
pub const MAX_BODY_SIZE: usize = 32 * 1024 * 1024; // 32 MB (P15: Bounded without cutting off large prompts)
pub const SOCKET_IDLE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60); // P15: Idle read timeout
pub const UPSTREAM_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);
pub const MAX_CONCURRENT_CONNECTIONS: usize = 128;

/// P16: Performs constant-time string comparison to prevent timing attacks on proxy auth tokens.
pub fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes().zip(b.bytes()).fold(0, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// P16: Validates Host header against loopback addresses or configured bind host (DNS rebinding guard).
pub fn is_allowed_host(host_header: &str, bind_addr: &str) -> bool {
    let host_trimmed = host_header.trim();
    let host_clean = if host_trimmed.starts_with('[') {
        if let Some(end_bracket) = host_trimmed.find(']') {
            &host_trimmed[..=end_bracket]
        } else {
            host_trimmed
        }
    } else {
        host_trimmed.split(':').next().unwrap_or(host_trimmed).trim()
    };

    let host_inner = host_clean.trim_matches(|c| c == '[' || c == ']');
    if host_clean == "localhost"
        || host_clean == "127.0.0.1"
        || host_clean == "::1"
        || host_clean == "[::1]"
        || host_inner == "::1"
    {
        return true;
    }
    if let Some((bind_host, _)) = bind_addr.rsplit_once(':') {
        let b = bind_host.trim_matches(|c| c == '[' || c == ']');
        if host_inner == b {
            return true;
        }
    }
    false
}

/// Decodes an HTTP/1.1 chunked transfer-encoded byte slice (RFC 7230 §4.1) into raw body bytes.
pub fn decode_chunked_body(mut input: &[u8]) -> Result<Vec<u8>, &'static str> {
    let mut decoded = Vec::new();
    loop {
        if input.is_empty() {
            return Err("Unexpected EOF in chunked body");
        }
        let nl = match input.windows(2).position(|w| w == b"\r\n") {
            Some(pos) => pos,
            None => return Err("Missing CRLF after chunk size"),
        };
        let size_str = std::str::from_utf8(&input[..nl]).map_err(|_| "Invalid UTF-8 in chunk size")?;
        let hex_size = size_str.split(';').next().unwrap_or("").trim();
        let chunk_len = usize::from_str_radix(hex_size, 16).map_err(|_| "Invalid hex in chunk size")?;

        if chunk_len == 0 {
            return Ok(decoded);
        }

        // Hardened: Guard against integer overflow and unbounded memory consumption (MAX_BODY_SIZE = 32MB)
        if chunk_len > MAX_BODY_SIZE || decoded.len().saturating_add(chunk_len) > MAX_BODY_SIZE {
            return Err("Chunked payload exceeds maximum allowable size");
        }

        let data_start = nl + 2;
        let data_end = match data_start.checked_add(chunk_len) {
            Some(end) => end,
            None => return Err("Chunk size overflow"),
        };
        if input.len() < data_end + 2 {
            return Err("Incomplete chunk data");
        }
        decoded.extend_from_slice(&input[data_start..data_end]);
        if &input[data_end..data_end + 2] != b"\r\n" {
            return Err("Missing CRLF after chunk data");
        }
        input = &input[data_end + 2..];
    }
}

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

/// Resolves the actual upstream target URL.
/// Supports 'auto' mode (dynamically routes Anthropic endpoints to api.anthropic.com,
/// OpenAI endpoints to api.openai.com, and Ollama endpoints to localhost:11434).
/// Also cleanses and normalizes path joining to prevent duplicate /v1 prefixes.
pub fn resolve_upstream_target(
    configured_upstream: &str,
    full_path: &str,
    headers: &[(String, String)],
) -> String {
    let clean_path = if full_path.is_empty() { "/" } else { full_path };
    let (path_no_query, _) = clean_path.split_once('?').unwrap_or((clean_path, ""));

    if configured_upstream.trim().is_empty() || configured_upstream.eq_ignore_ascii_case("auto") {
        let has_anthropic_header = headers.iter().any(|(k, _)| {
            k.eq_ignore_ascii_case("x-api-key") || k.eq_ignore_ascii_case("anthropic-version")
        });

        if path_no_query.starts_with("/v1/messages")
            || path_no_query.starts_with("/v1/complete")
            || has_anthropic_header
        {
            format!("https://api.anthropic.com{}", clean_path)
        } else if path_no_query.starts_with("/api/") {
            format!("http://localhost:11434{}", clean_path)
        } else {
            format!("https://api.openai.com{}", clean_path)
        }
    } else {
        let base = configured_upstream.trim_end_matches('/');
        if base.ends_with("/v1") {
            if clean_path == "/v1" {
                base.to_string()
            } else if clean_path.starts_with("/v1/") {
                format!("{}{}", base, &clean_path[3..])
            } else if !clean_path.starts_with('/') {
                format!("{}/{}", base, clean_path)
            } else {
                format!("{}{}", base, clean_path)
            }
        } else if !clean_path.starts_with('/') {
            format!("{}/{}", base, clean_path)
        } else {
            format!("{}{}", base, clean_path)
        }
    }
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
    let config = ProxyConfig {
        bind_addr: bind_addr.to_string(),
        upstream_url: upstream_url.to_string(),
        allow_remote,
        auth_token: auth_token.map(|t| t.to_string()),
        max_hourly_tokens: None,
        max_retries: 3,
        auto_retry_429: true,
    };
    run_reverse_proxy_with_config(config).await
}

/// Runs the AI Gateway Reverse Proxy with structured configuration including circuit breakers and rate limit mitigators.
pub async fn run_reverse_proxy_with_config(config: ProxyConfig) -> anyhow::Result<()> {
    let loopback = is_loopback(&config.bind_addr);
    if !loopback {
        if !config.allow_remote {
            return Err(anyhow::anyhow!(
                "Security Violation: Binding to non-loopback address '{}' requires explicit '--allow-remote' flag.",
                config.bind_addr
            ));
        }
        if config.auth_token.as_ref().map(|t| t.trim().is_empty()).unwrap_or(true) {
            return Err(anyhow::anyhow!(
                "Security Violation: Remote proxy mode on '{}' requires an authentication token via '--proxy-token' or 'TOKENECTOMY_PROXY_TOKEN'.",
                config.bind_addr
            ));
        }
    }

    let listener = TcpListener::bind(&config.bind_addr).await?;
    let client = reqwest::Client::builder()
        .tcp_nodelay(true)
        .timeout(UPSTREAM_TIMEOUT)
        .build()?;
    let client = Arc::new(client);
    let upstream = Arc::new(config.upstream_url.trim_end_matches('/').to_string());
    let required_token = config.auth_token.as_ref().map(|t| Arc::new(t.trim().to_string()));
    let semaphore = Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_CONNECTIONS));
    let analyzer_state = Arc::new(crate::analyzer::AppState::new());
    let metrics = Arc::new(ProxyMetrics::new());
    let prompt_cache: SharedResponseCache = Arc::new(RwLock::new(HashMap::new()));
    let bind_addr_arc = Arc::new(config.bind_addr.clone());
    let config_arc = Arc::new(config.clone());

    println!("⚡ Tokenectomy AI Gateway Proxy active on http://{}", config.bind_addr);
    println!("📊 Real-Time FinOps Dashboard: http://{}/dashboard", config.bind_addr);
    println!("📈 Prometheus / JSON Metrics: http://{}/v1/metrics", config.bind_addr);
    if upstream.eq_ignore_ascii_case("auto") {
        println!("🔀 Smart Upstream Routing: AUTO (Anthropic -> api.anthropic.com, OpenAI -> api.openai.com, Ollama -> localhost:11434)");
    } else {
        println!("🔗 Forwarding to upstream: {}", upstream);
    }
    if let Some(limit) = config.max_hourly_tokens {
        println!("🛑 Safety Circuit Breaker: ACTIVE (Cap: {} tokens/hour)", limit);
    }
    if config.auto_retry_429 {
        println!("🔄 Resilient 429 Mitigator: ACTIVE (Max retries: {})", config.max_retries);
    }
    if !loopback {
        println!("🔒 Remote mode ACTIVE (Protected with mandatory bearer authentication)");
    }
    println!("🛡️ Active filters: Zero-Leak Redaction + Polyglot Framework Surgery + Prompt Cache");

    loop {
        let (mut socket, peer_addr) = listener.accept().await?;
        let _ = socket.set_nodelay(true);
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
        let analyzer_clone = Arc::clone(&analyzer_state);
        let metrics_clone = Arc::clone(&metrics);
        let prompt_cache_clone = Arc::clone(&prompt_cache);
        let bind_addr_clone = Arc::clone(&bind_addr_arc);
        let config_clone = Arc::clone(&config_arc);

        tokio::spawn(async move {
            let _permit = permit;
            let handler = async {
                let mut buf = vec![0u8; 16384];
                let mut total_read = 0;

                // Read HTTP request headers with MAX_HEADER_SIZE bound and idle timeout
                while total_read < MAX_HEADER_SIZE {
                    if buf.len() <= total_read {
                        buf.resize(buf.len() * 2, 0);
                    }
                    match tokio::time::timeout(SOCKET_IDLE_TIMEOUT, socket.read(&mut buf[total_read..])).await {
                        Ok(Ok(0)) => break,
                        Ok(Ok(n)) => {
                            total_read += n;
                            if buf[..total_read].windows(4).any(|w| w == b"\r\n\r\n") {
                                break;
                            }
                        }
                        Ok(Err(e)) => {
                            log::error!("Socket read error from {}: {}", peer_addr, e);
                            return;
                        }
                        Err(_) => {
                            log::warn!("Socket idle timeout waiting for headers from {}", peer_addr);
                            return;
                        }
                    }
                }

                if total_read == 0 {
                    return;
                }

                let header_end = match buf[..total_read].windows(4).position(|w| w == b"\r\n\r\n") {
                    Some(idx) => idx,
                    None => {
                        let resp = "HTTP/1.1 431 Request Header Fields Too Large\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"error\":{\"message\":\"Headers exceed 64KB limit\",\"code\":431}}\r\n";
                        let _ = socket.write_all(resp.as_bytes()).await;
                        return;
                    }
                };

                let body_start = header_end + 4;
                let req_str = String::from_utf8_lossy(&buf[..header_end]);
                let raw_headers = req_str.as_ref();
                let mut lines = raw_headers.split("\r\n");
                let req_line = lines.next().unwrap_or("");
                let parts: Vec<&str> = req_line.split_whitespace().collect();

                if parts.len() < 2 {
                    return;
                }

                let method = parts[0];
                let full_path = parts[1];
                let (raw_path_no_query, _) = full_path.split_once('?').unwrap_or((full_path, ""));
                let path = if raw_path_no_query.len() > 1 && raw_path_no_query.ends_with('/') {
                    raw_path_no_query.trim_end_matches('/')
                } else {
                    raw_path_no_query
                };

                // Browser & Agent CORS Preflight
                if method == "OPTIONS" {
                    let resp = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS, HEAD\r\nAccess-Control-Allow-Headers: *\r\nAccess-Control-Max-Age: 86400\r\nConnection: close\r\n\r\n";
                    let _ = socket.write_all(resp.as_bytes()).await;
                    return;
                }

                // Health check endpoint
                if path == "/health" || path == "/v1/health" {
                    let body = format!(
                        "{{\"status\":\"ok\",\"service\":\"tokenectomy-gateway\",\"version\":\"{}\"}}\r\n",
                        env!("CARGO_PKG_VERSION")
                    );
                    let resp = if method == "HEAD" {
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        )
                    } else {
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        )
                    };
                    let _ = socket.write_all(resp.as_bytes()).await;
                    return;
                }

                let req_host = raw_headers
                    .lines()
                    .skip(1)
                    .find_map(|l| {
                        let (k, v) = l.split_once(':')?;
                        if k.trim().eq_ignore_ascii_case("host") {
                            Some(v.trim())
                        } else {
                            None
                        }
                    })
                    .unwrap_or("127.0.0.1");

                // P16: Prometheus / JSON Metrics API endpoint with DNS rebinding protection
                if path == "/v1/metrics" || path == "/metrics" {
                    if !is_allowed_host(req_host, &bind_addr_clone) {
                        let resp = "HTTP/1.1 403 Forbidden\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"error\":{\"message\":\"Forbidden: Host header mismatch (DNS rebinding guard)\",\"code\":403}}\r\n";
                        let _ = socket.write_all(resp.as_bytes()).await;
                        return;
                    }
                    let metrics_json = metrics_clone.to_json().to_string();
                    let resp = if method == "HEAD" {
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            metrics_json.len()
                        )
                    } else {
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            metrics_json.len(),
                            metrics_json
                        )
                    };
                    let _ = socket.write_all(resp.as_bytes()).await;
                    return;
                }

                // P16: Embedded FinOps Dashboard UI with DNS rebinding protection
                if (method == "GET" || method == "HEAD") && (path == "/dashboard" || path == "/" || path == "/ui") {
                    if !is_allowed_host(req_host, &bind_addr_clone) {
                        let resp = "HTTP/1.1 403 Forbidden\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"error\":{\"message\":\"Forbidden: Host header mismatch (DNS rebinding guard)\",\"code\":403}}\r\n";
                        let _ = socket.write_all(resp.as_bytes()).await;
                        return;
                    }
                    let html = crate::dashboard::render_dashboard_html();
                    let resp = if method == "HEAD" {
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            html.len()
                        )
                    } else {
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            html.len(),
                            html
                        )
                    };
                    let _ = socket.write_all(resp.as_bytes()).await;
                    return;
                }

                // Extract Content-Length & Authorization, and collect client headers for upstream forwarding
                let mut content_length = 0;
                let mut is_chunked = false;
                let mut client_auth = None;
                let mut forward_headers = Vec::new();

                for h in raw_headers.lines().skip(1) {
                    if let Some((k, v)) = h.split_once(':') {
                        let k_clean = k.trim();
                        let v_clean = v.trim();
                        let k_lower = k_clean.to_ascii_lowercase();

                        if k_lower == "content-length" {
                            content_length = v_clean.parse::<usize>().unwrap_or(0);
                        } else if k_lower == "transfer-encoding" && v_clean.to_ascii_lowercase().contains("chunked") {
                            is_chunked = true;
                        } else if k_lower == "authorization" {
                            client_auth = Some(v_clean.to_string());
                        } else if k_lower == "x-api-key" && client_auth.is_none() {
                            client_auth = Some(format!("Bearer {}", v_clean));
                        }

                        // RFC 7230 §6.1: Strip hop-by-hop headers and Host header
                        if k_lower != "content-length"
                            && k_lower != "connection"
                            && k_lower != "transfer-encoding"
                            && k_lower != "keep-alive"
                            && k_lower != "proxy-connection"
                            && k_lower != "upgrade"
                            && k_lower != "host"
                        {
                            forward_headers.push((k_clean.to_string(), v_clean.to_string()));
                        }
                    }
                }

                // P16: Enforce authentication for remote mode with constant-time compare
                if let Some(expected_token) = &token_clone {
                    let is_authed = match &client_auth {
                        Some(auth) => {
                            let token_part = auth.strip_prefix("Bearer ").unwrap_or(auth.as_str()).trim();
                            constant_time_compare(token_part, expected_token.as_str())
                        }
                        None => false,
                    };
                    if !is_authed {
                        let resp = "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"error\":{\"message\":\"Unauthorized: Missing or invalid proxy authorization token\",\"code\":401}}\r\n";
                        let _ = socket.write_all(resp.as_bytes()).await;
                        return;
                    }
                }

                // Read body (either chunked or Content-Length framed) with idle timeout
                let body_bytes = if is_chunked {
                    let mut raw_chunked_bytes = buf[body_start..total_read].to_vec();
                    while raw_chunked_bytes.len() < MAX_BODY_SIZE {
                        if raw_chunked_bytes.windows(5).any(|w| w == b"0\r\n\r\n") {
                            break;
                        }
                        let mut temp = vec![0u8; 8192];
                        match tokio::time::timeout(SOCKET_IDLE_TIMEOUT, socket.read(&mut temp)).await {
                            Ok(Ok(0)) => break,
                            Ok(Ok(n)) => {
                                raw_chunked_bytes.extend_from_slice(&temp[..n]);
                                if raw_chunked_bytes.windows(5).any(|w| w == b"0\r\n\r\n") {
                                    break;
                                }
                            }
                            _ => break,
                        }
                    }

                    if raw_chunked_bytes.len() >= MAX_BODY_SIZE {
                        let resp = "HTTP/1.1 413 Payload Too Large\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"error\":{\"message\":\"Payload exceeds configured body limit\",\"code\":413}}\r\n";
                        let _ = socket.write_all(resp.as_bytes()).await;
                        return;
                    }

                    match decode_chunked_body(&raw_chunked_bytes) {
                        Ok(b) => b,
                        Err(e) => {
                            let resp = format!(
                                "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{{\"error\":{{\"message\":\"Malformed chunked transfer body: {}\",\"code\":400}}}}\r\n",
                                e
                            );
                            let _ = socket.write_all(resp.as_bytes()).await;
                            return;
                        }
                    }
                } else {
                    if content_length > MAX_BODY_SIZE {
                        let resp = "HTTP/1.1 413 Payload Too Large\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"error\":{\"message\":\"Payload exceeds configured body limit\",\"code\":413}}\r\n";
                        let _ = socket.write_all(resp.as_bytes()).await;
                        return;
                    }

                    let mut bytes = buf[body_start..total_read].to_vec();
                    while bytes.len() < content_length {
                        let to_read = (content_length - bytes.len()).min(16384);
                        let mut temp = vec![0u8; to_read];
                        match tokio::time::timeout(SOCKET_IDLE_TIMEOUT, socket.read(&mut temp)).await {
                            Ok(Ok(0)) => break,
                            Ok(Ok(n)) => bytes.extend_from_slice(&temp[..n]),
                            _ => break,
                        }
                    }
                    bytes
                };

                // Handle /v1/analyze or /analyze directly on the gateway (#17, #18, #19, #20)
                if method == "POST" && (path == "/v1/analyze" || path == "/analyze") {
                    if let Ok(analyze_req) = serde_json::from_slice::<crate::analyzer::AnalyzeRequest>(&body_bytes) {
                        match crate::analyzer::analyze_source(&analyzer_clone, &analyze_req.language, &analyze_req.code) {
                            Ok(resp) => {
                                eprintln!(
                                    "⚡ [Proxy] /v1/analyze from {}: {} findings, {:.2}ms",
                                    peer_addr, resp.total_findings, resp.duration_ms
                                );
                                let res_json = serde_json::to_string(&resp).unwrap_or_default();
                                let resp_http = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                    res_json.len(),
                                    res_json
                                );
                                let _ = socket.write_all(resp_http.as_bytes()).await;
                                return;
                            }
                            Err(e) => {
                                let err_json = serde_json::json!({
                                    "version": "v1",
                                    "status": "error",
                                    "error": e
                                }).to_string();
                                let resp_http = format!(
                                    "HTTP/1.1 422 Unprocessable Entity\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                    err_json.len(),
                                    err_json
                                );
                                let _ = socket.write_all(resp_http.as_bytes()).await;
                                return;
                            }
                        }
                    } else {
                        let err_json = serde_json::json!({
                            "error": "Invalid JSON payload for /v1/analyze"
                        }).to_string();
                        let resp_http = format!(
                            "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            err_json.len(),
                            err_json
                        );
                        let _ = socket.write_all(resp_http.as_bytes()).await;
                        return;
                    }
                }

                // Process and sanitize body if JSON
                let mut final_body = body_bytes;
                if let Ok(json) = serde_json::from_slice::<Value>(&final_body) {
                    let (sanitized, stats) = sanitize_prompt_payload(&json);
                    metrics_clone.record(stats.raw_chars, stats.sanitized_chars, stats.secrets_redacted);
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
                } else {
                    metrics_clone.record_request();
                }

                // Forward to upstream with complete query string preserved and dynamic auto-routing
                let mut target_url = resolve_upstream_target(&upstream_clone, full_path, &forward_headers);

                let transpile_dir = crate::transpiler::detect_transpile_direction(
                    path,
                    &target_url,
                    &forward_headers,
                );

                let target_model_override = forward_headers.iter().find_map(|(k, v)| {
                    if k.eq_ignore_ascii_case("x-tokenectomy-target-model") || k.eq_ignore_ascii_case("x-target-model") {
                        Some(v.clone())
                    } else {
                        None
                    }
                }).or_else(|| std::env::var("TOKENECTOMY_TARGET_MODEL").ok());

                if transpile_dir == crate::transpiler::TranspileDirection::AnthropicToOpenAi {
                    // Rewrite target endpoint to /v1/chat/completions preserving query string
                    target_url = crate::transpiler::rewrite_transpiled_url(&target_url, "/v1/chat/completions");

                    if let Ok(json_body) = serde_json::from_slice::<Value>(&final_body) {
                        if let Ok(openai_body) = crate::transpiler::anthropic_to_openai_request(&json_body, target_model_override.as_deref()) {
                            if let Ok(transpiled_bytes) = serde_json::to_vec(&openai_body) {
                                final_body = transpiled_bytes;
                            }
                        }
                    }

                    let has_auth = forward_headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("authorization"));
                    if !has_auth {
                        if let Some((_, val)) = forward_headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("x-api-key")) {
                            forward_headers.push(("Authorization".to_string(), format!("Bearer {}", val)));
                        }
                    }
                } else if transpile_dir == crate::transpiler::TranspileDirection::OpenAiToAnthropic {
                    // Rewrite target endpoint to /v1/messages preserving query string
                    target_url = crate::transpiler::rewrite_transpiled_url(&target_url, "/v1/messages");

                    if let Ok(json_body) = serde_json::from_slice::<Value>(&final_body) {
                        if let Ok(ant_body) = crate::transpiler::openai_to_anthropic_request(&json_body, target_model_override.as_deref()) {
                            if let Ok(transpiled_bytes) = serde_json::to_vec(&ant_body) {
                                final_body = transpiled_bytes;
                            }
                        }
                    }

                    let has_ant_ver = forward_headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("anthropic-version"));
                    if !has_ant_ver {
                        forward_headers.push(("anthropic-version".to_string(), "2023-06-01".to_string()));
                    }
                    if let Some((_, val)) = forward_headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("authorization")) {
                        let token = val.strip_prefix("Bearer ").unwrap_or(val).trim();
                        forward_headers.push(("x-api-key".to_string(), token.to_string()));
                    }
                }

                // Check safety circuit breaker on hourly token budget
                let est_inbound_tokens = (final_body.len() as u64) / 4;
                if !metrics_clone.check_and_record_hourly_tokens(est_inbound_tokens, config_clone.max_hourly_tokens) {
                    eprintln!(
                        "🚨 [Circuit Breaker TRIPPED] Hourly token limit exceeded ({} tokens). Blocking request from {} to prevent runaway loop.",
                        est_inbound_tokens, peer_addr
                    );
                    let trip_err = serde_json::json!({
                        "error": {
                            "message": "Tokenectomy Circuit Breaker TRIPPED: Hourly token limit reached. Halting request to prevent runaway agent loops and unexpected billing.",
                            "type": "circuit_breaker_tripped",
                            "code": 429
                        }
                    }).to_string();
                    let resp = format!(
                        "HTTP/1.1 429 Too Many Requests\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nX-Tokenectomy-Circuit-Breaker: TRIPPED\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        trip_err.len(),
                        trip_err
                    );
                    let _ = socket.write_all(resp.as_bytes()).await;
                    return;
                }

                // Check zero-cost prompt cache for non-streaming completion requests (respecting Cache-Control: no-cache)
                let is_stream = if let Ok(json_val) = serde_json::from_slice::<Value>(&final_body) {
                    json_val.get("stream").and_then(|s| s.as_bool()).unwrap_or(false)
                } else {
                    false
                };

                let no_cache = raw_headers.lines().any(|l| {
                    if let Some((k, v)) = l.split_once(':') {
                        k.trim().eq_ignore_ascii_case("cache-control") && v.to_ascii_lowercase().contains("no-cache")
                    } else {
                        false
                    }
                });

                let cache_eligible = method == "POST" && !is_stream && !no_cache && (path.ends_with("/chat/completions") || path.ends_with("/messages"));
                let cache_key = if cache_eligible {
                    let mut hasher = Sha256::new();
                    hasher.update(target_url.as_bytes());
                    hasher.update(b":");
                    hasher.update(&final_body);
                    let result = hasher.finalize();
                    let mut hash_str = String::with_capacity(64);
                    for byte in result {
                        hash_str.push_str(&format!("{:02x}", byte));
                    }
                    Some(hash_str)
                } else {
                    None
                };

                if let Some(ref key) = cache_key {
                    let cache_read = prompt_cache_clone.read().await;
                    if let Some(entry) = cache_read.get(key) {
                        if entry.timestamp.elapsed() < entry.ttl {
                            metrics_clone.record_cache_hit(entry.estimated_tokens);
                            metrics_clone.record_latency(0);
                            eprintln!(
                                "⚡ [Gateway Cache HIT] Saved {} tokens for {}",
                                entry.estimated_tokens, peer_addr
                            );

                            let mut head = format!("HTTP/1.1 {} {}\r\n", entry.status, entry.reason);
                            head.push_str("Access-Control-Allow-Origin: *\r\n");
                            for (k, v) in &entry.headers {
                                head.push_str(&format!("{}: {}\r\n", k, v));
                            }
                            head.push_str("X-Tokenectomy-Cache: HIT\r\n");
                            head.push_str(&format!("X-Tokenectomy-Saved-Tokens: {}\r\n", entry.estimated_tokens));
                            head.push_str(&format!("Content-Length: {}\r\n", entry.body.len()));
                            head.push_str("Connection: close\r\n\r\n");

                            let _ = socket.write_all(head.as_bytes()).await;
                            let _ = socket.write_all(&entry.body).await;
                            let _ = socket.flush().await;
                            return;
                        }
                    }
                }

                let req_start = Instant::now();
                let mut retries = 0;
                let max_retries = if config_clone.auto_retry_429 { config_clone.max_retries } else { 0 };

                let upstream_send_result = loop {
                    let mut req_builder = match method {
                        "POST" => client_clone.post(&target_url),
                        "GET" => client_clone.get(&target_url),
                        _ => client_clone.request(
                            reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET),
                            &target_url,
                        ),
                    };

                    for (hk, hv) in &forward_headers {
                        req_builder = req_builder.header(hk, hv);
                    }

                    if let Some(ref auth) = client_auth {
                        req_builder = req_builder.header("Authorization", auth);
                    }
                    if !final_body.is_empty() {
                        req_builder = req_builder
                            .header("Content-Type", "application/json")
                            .body(final_body.clone());
                    }

                    match req_builder.send().await {
                        Ok(resp) => {
                            let status = resp.status();
                            if status.as_u16() == 429 && retries < max_retries {
                                retries += 1;
                                metrics_clone.record_rate_limit_mitigated();

                                let wait_secs = resp.headers()
                                    .get("retry-after")
                                    .and_then(|h| h.to_str().ok())
                                    .and_then(|s| s.parse::<u64>().ok())
                                    .unwrap_or_else(|| (2u64.pow(retries as u32)).min(20));

                                eprintln!(
                                    "⚠️ [Gateway 429 Mitigator] Upstream rate limit hit (429) for {}. Pausing for {}s before retry ({}/{})...",
                                    peer_addr, wait_secs, retries, max_retries
                                );

                                tokio::time::sleep(Duration::from_secs(wait_secs.min(30))).await;
                                continue;
                            }
                            break Ok(resp);
                        }
                        Err(e) => {
                            break Err(e);
                        }
                    }
                };

                let elapsed_ms = req_start.elapsed().as_millis() as u64;
                metrics_clone.record_latency(elapsed_ms);

                match upstream_send_result {
                    Ok(mut upstream_resp) => {
                        let status = upstream_resp.status();
                        let is_event_stream = upstream_resp.headers()
                            .get("content-type")
                            .and_then(|ct| ct.to_str().ok())
                            .map(|ct| ct.contains("text/event-stream"))
                            .unwrap_or(false);

                        let mut captured_headers = Vec::new();
                        let mut captured_body = Vec::new();

                        if !is_event_stream && transpile_dir != crate::transpiler::TranspileDirection::None {
                            let mut raw_upstream_body = Vec::new();
                            while let Ok(Ok(Some(chunk))) = tokio::time::timeout(SOCKET_IDLE_TIMEOUT, upstream_resp.chunk()).await {
                                if raw_upstream_body.len() < MAX_BODY_SIZE {
                                    raw_upstream_body.extend_from_slice(&chunk);
                                }
                            }

                            let mut transformed_body = raw_upstream_body.clone();
                            if let Ok(resp_json) = serde_json::from_slice::<Value>(&raw_upstream_body) {
                                if transpile_dir == crate::transpiler::TranspileDirection::AnthropicToOpenAi {
                                    if let Ok(ant_resp) = crate::transpiler::openai_to_anthropic_response(&resp_json, None) {
                                        if let Ok(bytes) = serde_json::to_vec(&ant_resp) {
                                            transformed_body = bytes;
                                        }
                                    }
                                } else if transpile_dir == crate::transpiler::TranspileDirection::OpenAiToAnthropic {
                                    if let Ok(openai_resp) = crate::transpiler::anthropic_to_openai_response(&resp_json) {
                                        if let Ok(bytes) = serde_json::to_vec(&openai_resp) {
                                            transformed_body = bytes;
                                        }
                                    }
                                }
                            }

                            let mut head = format!("HTTP/1.1 {} {}\r\n", status.as_u16(), status.canonical_reason().unwrap_or(""));
                            head.push_str("Access-Control-Allow-Origin: *\r\n");
                            head.push_str("Content-Type: application/json\r\n");
                            head.push_str(&format!("Content-Length: {}\r\n", transformed_body.len()));
                            head.push_str("X-Tokenectomy-Transpiled: true\r\n");
                            head.push_str("Connection: close\r\n\r\n");

                            let _ = socket.write_all(head.as_bytes()).await;
                            let _ = socket.write_all(&transformed_body).await;
                            let _ = socket.flush().await;
                            captured_body = transformed_body;
                        } else {
                            let mut head = format!("HTTP/1.1 {} {}\r\n", status.as_u16(), status.canonical_reason().unwrap_or(""));
                            head.push_str("Access-Control-Allow-Origin: *\r\n");
                            if retries > 0 {
                                head.push_str(&format!("X-Tokenectomy-Rate-Limits-Mitigated: {}\r\n", retries));
                            }

                            for (k, v) in upstream_resp.headers() {
                                let k_lower = k.as_str().to_ascii_lowercase();
                                // RFC 7230 §6.1: Strip hop-by-hop headers to prevent chunked framing mismatches
                                if k_lower == "transfer-encoding"
                                    || k_lower == "connection"
                                    || k_lower == "keep-alive"
                                    || k_lower == "proxy-connection"
                                    || k_lower == "upgrade"
                                {
                                    continue;
                                }
                                let v_str = v.to_str().unwrap_or("");
                                head.push_str(&format!("{}: {}\r\n", k.as_str(), v_str));
                                captured_headers.push((k.as_str().to_string(), v_str.to_string()));
                            }

                            if cache_eligible {
                                head.push_str("X-Tokenectomy-Cache: MISS\r\n");
                            }
                            head.push_str("Connection: close\r\n\r\n");
                            let _ = socket.write_all(head.as_bytes()).await;

                            // P15: Stream chunk by chunk with idle timeout
                            while let Ok(Ok(Some(chunk))) = tokio::time::timeout(SOCKET_IDLE_TIMEOUT, upstream_resp.chunk()).await {
                                if cache_eligible && !is_event_stream && captured_body.len() < 4 * 1024 * 1024 {
                                    captured_body.extend_from_slice(&chunk);
                                }
                                if socket.write_all(&chunk).await.is_err() {
                                    break;
                                }
                                let _ = socket.flush().await;
                            }
                            let _ = socket.flush().await;
                        }

                        // Store in zero-cost prompt cache on success
                        if let Some(key) = cache_key {
                            if status.is_success() && !is_event_stream && !captured_body.is_empty() {
                                let est_tokens = (final_body.len() + captured_body.len()) as u64 / 4;
                                let mut cache_write = prompt_cache_clone.write().await;
                                if cache_write.len() >= 1000 {
                                    cache_write.clear();
                                }
                                cache_write.insert(key, CachedCompletion {
                                    status: status.as_u16(),
                                    reason: status.canonical_reason().unwrap_or("OK").to_string(),
                                    headers: captured_headers,
                                    body: captured_body,
                                    timestamp: Instant::now(),
                                    ttl: Duration::from_secs(300),
                                    estimated_tokens: est_tokens,
                                });
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

            // P15: Connection lifetime bounded by active stream idle reads rather than arbitrary total connection cutoff
            handler.await;
        });
    }
}
