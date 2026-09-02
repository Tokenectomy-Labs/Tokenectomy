use super::AiProvider;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct AnthropicProvider {
    api_key: String,
    client: Client,
}

impl AnthropicProvider {
    pub fn new(config_key: Option<String>) -> Result<Self> {
        let api_key = config_key.or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .ok_or_else(|| anyhow!("ANTHROPIC_API_KEY not set in environment or config"))?;
        Ok(Self {
            api_key,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()?,
        })
    }
}

#[async_trait]
impl AiProvider for AnthropicProvider {
    async fn explain(&self, log: &str, context: &str) -> Result<String> {
        let prompt = format!(
            "Analyze the following error log and the extracted source code context. Explain the root cause clearly and provide a concrete solution.\n\nError Log:\n{}\n\nContext:\n{}",
            log, context
        );

        let body = json!({
            "model": "claude-3-5-sonnet-20240620",
            "max_tokens": 1024,
            "messages": [
                {"role": "user", "content": prompt}
            ]
        });

        let response = self.client.post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Anthropic API error: {}", error_text));
        }

        let resp_json: serde_json::Value = response.json().await?;
        let content = resp_json["content"][0]["text"]
            .as_str()
            .unwrap_or("Failed to parse response")
            .to_string();

        Ok(content)
    }
}
