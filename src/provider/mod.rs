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

pub struct MockProvider;

#[async_trait]
impl AiProvider for MockProvider {
    async fn explain(&self, log: &str, context: &str) -> Result<String> {
        Ok(format!(
            "[MOCK RESPONSE]\nRoot Cause: An error was found in the log related to the source code above.\nSolution: Ensure all variables are properly initialized and the logic handles edge cases."
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
