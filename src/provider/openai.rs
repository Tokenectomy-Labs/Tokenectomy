use super::AiProvider;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct OpenAiProvider {
    api_key: String,
    client: Client,
}

impl OpenAiProvider {
    pub fn new(config_key: Option<String>) -> Result<Self> {
        let api_key = config_key.or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .ok_or_else(|| anyhow!("OPENAI_API_KEY not set in environment or config"))?;
        Ok(Self {
            api_key,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()?,
        })
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    async fn explain(&self, log: &str, context: &str) -> Result<String> {
        let prompt = format!(
            "You are a helpful expert developer. Analyze the following error log and the extracted source code context. Explain the root cause of the error clearly and concisely, and provide a concrete solution to fix it.\n\nError Log:\n{}\n\nContext:\n{}",
            log, context
        );

        let body = json!({
            "model": "gpt-4o",
            "messages": [
                {"role": "system", "content": "You are a specialized debugging assistant. Format your response with 'Root Cause:' followed by 'Solution:'."},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.2
        });

        let response = self.client.post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI API error: {}", error_text));
        }

        let resp_json: serde_json::Value = response.json().await?;
        let content = resp_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("Failed to parse response")
            .to_string();

        Ok(content)
    }
}
