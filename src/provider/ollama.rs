use super::AiProvider;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct OllamaProvider {
    base_url: String,
    client: Client,
}

impl OllamaProvider {
    pub fn new(config_url: Option<String>) -> Self {
        let base_url = config_url
            .or_else(|| std::env::var("OLLAMA_BASE_URL").ok())
            .unwrap_or_else(|| "http://localhost:11434".to_string());
        
        // VULN-04: SSRF protection — warn if non-localhost URL is configured
        if !base_url.contains("localhost") && !base_url.contains("127.0.0.1") && !base_url.contains("::1") {
            eprintln!("⚠️  WARNING: Ollama URL points to a remote server: {}", base_url);
            eprintln!("   Your error logs and source code will be sent to this server.");
            eprintln!("   If this is unintended, check your .tokenectomy.toml or OLLAMA_BASE_URL.");
        }

        Self {
            base_url,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }
}

#[async_trait]
impl AiProvider for OllamaProvider {
    async fn explain(&self, log: &str, context: &str) -> Result<String> {
        let prompt = format!(
            "Analyze the following error log and the extracted source code context. Explain the root cause and provide a solution.\n\nError Log:\n{}\n\nContext:\n{}",
            log, context
        );

        let body = json!({
            "model": "llama3", // Default fallback model
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "stream": false
        });

        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));
        
        let response = self.client.post(&url)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Ollama API error: {}", error_text));
        }

        let resp_json: serde_json::Value = response.json().await?;
        let content = resp_json["message"]["content"]
            .as_str()
            .unwrap_or("Failed to parse response")
            .to_string();

        Ok(content)
    }
}
