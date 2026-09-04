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
    async fn explain(&self, log: &str, _context: &str) -> Result<String> {
        if log.contains("server.py") || log.contains("ZeroDivisionError") {
            Ok(r#"💡 Root Cause Analysis:
ZeroDivisionError detected in server.py:10
The variable `active_users` evaluates to 0, causing division by zero inside `calculate_metrics()`.

🔧 Recommended Fix:
Guard division by zero with a fallback default:
```python
per_user = total_tokens / max(active_users, 1)
```"#.to_string())
        } else {
            Ok(r#"💡 Root Cause Analysis:
Unhandled exception detected in application execution.

🔧 Recommended Fix:
Validate input parameters and handle boundary conditions before execution."#.to_string())
        }
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
