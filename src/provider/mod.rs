pub mod anthropic;
pub mod ollama;
pub mod openai;

use crate::cli::ProviderChoice;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait AiProvider {
    async fn explain(&self, log: &str, context: &str) -> Result<String>;
}

/// Offline testing / dry-run provider. Does not connect to an external LLM.
pub struct MockProvider;

#[async_trait]
impl AiProvider for MockProvider {
    async fn explain(&self, log: &str, context: &str) -> Result<String> {
        let first_line = log.lines().find(|l| !l.trim().is_empty()).unwrap_or("Unknown error");
        Ok(format!(
            "💡 [OFFLINE TEST MOCK — No LLM Connected]\n\
             Root Cause:\n\
             Diagnostic test run for: {}\n\n\
             Source Context Extracted:\n\
             {} lines of local code provided.\n\n\
             Solution:\n\
             Configure an active LLM provider (Ollama, OpenAI, or Anthropic) in ~/.tokenectomy.toml or via --provider.",
            first_line.trim(),
            context.lines().count()
        ))
    }
}

pub fn get_provider(choice: ProviderChoice, local_only: bool, config: &crate::config::AppConfig) -> Result<Box<dyn AiProvider>> {
    if local_only {
        return Ok(Box::new(ollama::OllamaProvider::new(config.ollama_base_url.clone())));
    }

    match choice {
        ProviderChoice::Openai => Ok(Box::new(openai::OpenAiProvider::new(config.openai_api_key.clone())?)),
        ProviderChoice::Anthropic => Ok(Box::new(anthropic::AnthropicProvider::new(config.anthropic_api_key.clone())?)),
        ProviderChoice::Ollama => Ok(Box::new(ollama::OllamaProvider::new(config.ollama_base_url.clone()))),
        ProviderChoice::Mock => Ok(Box::new(MockProvider)),
    }
}
