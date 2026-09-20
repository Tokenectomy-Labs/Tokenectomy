use crate::proxy::{run_reverse_proxy_with_config, ProxyConfig};
use anyhow::{Context, Result};
use colored::*;
use std::process::Stdio;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::process::Command;

/// Checks if a port is available on localhost, or finds the next available port.
pub async fn find_available_port(preferred: u16) -> u16 {
    if TcpListener::bind(format!("127.0.0.1:{}", preferred)).await.is_ok() {
        return preferred;
    }
    for port in (preferred + 1)..=(preferred + 30) {
        if TcpListener::bind(format!("127.0.0.1:{}", port)).await.is_ok() {
            return port;
        }
    }
    preferred
}

/// Runs a command with the Tokenectomy AI Gateway automatically spawned in the background,
/// injecting transparent environment variables (ANTHROPIC_BASE_URL, OPENAI_BASE_URL, etc.).
pub async fn run_wrap_command(
    cmd_parts: Vec<String>,
    proxy_bind: Option<String>,
    upstream_url: Option<String>,
    max_hourly_tokens: Option<u64>,
    max_retries: usize,
    auto_retry_429: bool,
) -> Result<i32> {
    if cmd_parts.is_empty() {
        anyhow::bail!("No command specified to wrap. Usage: razor wrap -- <command> [args...]");
    }

    let raw_bind = proxy_bind.unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let (bind_ip, default_port) = if let Some((ip, port_str)) = raw_bind.split_once(':') {
        (ip.to_string(), port_str.parse::<u16>().unwrap_or(8080))
    } else {
        ("127.0.0.1".to_string(), 8080)
    };

    let actual_port = find_available_port(default_port).await;
    let actual_bind = format!("{}:{}", bind_ip, actual_port);
    let upstream = upstream_url.unwrap_or_else(|| "auto".to_string());

    let proxy_config = ProxyConfig {
        bind_addr: actual_bind.clone(),
        upstream_url: upstream.clone(),
        allow_remote: false,
        auth_token: None,
        max_hourly_tokens,
        max_retries,
        auto_retry_429,
    };

    // 1. Spawn the proxy server as a background Tokio task
    let server_config = proxy_config.clone();
    let server_task = tokio::spawn(async move {
        if let Err(e) = run_reverse_proxy_with_config(server_config).await {
            log::error!("Tokenectomy wrapper proxy exited: {}", e);
        }
    });

    // 2. Poll until gateway health check succeeds (bounded to 3 seconds)
    let health_url = format!("http://{}/v1/health", actual_bind);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(150))
        .build()?;

    let mut is_ready = false;
    let start_wait = std::time::Instant::now();
    while start_wait.elapsed() < Duration::from_secs(3) {
        if let Ok(resp) = client.get(&health_url).send().await {
            if resp.status().is_success() {
                is_ready = true;
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    if !is_ready {
        server_task.abort();
        anyhow::bail!("Failed to start background Tokenectomy AI Gateway on {}", actual_bind);
    }

    let gateway_url = format!("http://{}", actual_bind);
    let openai_gateway_url = format!("http://{}/v1", actual_bind);

    // 3. User-friendly terminal banner
    eprintln!("{}", "┌─────────────────────────────────────────────────────────────┐".yellow());
    eprintln!("│ {} │", format!("{:^59}", "⚡ TOKENECTOMY RAZOR AI GATEWAY WRAPPER").yellow().bold());
    eprintln!("{}", "├─────────────────────────────────────────────────────────────┤".yellow());
    eprintln!("│  Gateway URL : {:<44} │", gateway_url.cyan().bold());
    eprintln!("│  Upstream    : {:<44} │", upstream.white());
    eprintln!("│  Injected Env: {:<44} │", "ANTHROPIC_BASE_URL, OPENAI_BASE_URL".green());
    eprintln!("│  Target Cmd  : {:<44} │", cmd_parts.join(" ").magenta().bold());
    eprintln!("{}", "└─────────────────────────────────────────────────────────────┘".yellow());

    // 4. Spawn child process with injected environment
    let executable = &cmd_parts[0];
    let args = &cmd_parts[1..];

    let mut cmd = Command::new(executable);
    cmd.args(args)
        .env("ANTHROPIC_BASE_URL", &gateway_url)
        .env("OPENAI_BASE_URL", &openai_gateway_url)
        .env("OPENAI_API_BASE", &openai_gateway_url)
        .env("OLLAMA_HOST", &gateway_url)
        .env("TOKENECTOMY_ACTIVE", "1")
        .env("TOKENECTOMY_GATEWAY_URL", &gateway_url)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    if let Ok(target_m) = std::env::var("TOKENECTOMY_TARGET_MODEL") {
        cmd.env("TOKENECTOMY_TARGET_MODEL", target_m);
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            server_task.abort();
            return Err(e).with_context(|| format!("Failed to spawn wrapped command: {}", executable));
        }
    };

    let exit_status = child.wait().await;
    server_task.abort();

    let code = match exit_status {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            log::error!("Error waiting on child process {}: {}", executable, e);
            1
        }
    };

    eprintln!(
        "\n{}",
        format!("⚡ [Tokenectomy] Wrapped execution finished with status code: {}", code).bright_black()
    );

    Ok(code)
}
