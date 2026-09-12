use serde::Deserialize;
use std::path::PathBuf;
use std::fs;
use crate::cli::ProviderChoice;

#[derive(Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct CustomRedactRuleConfig {
    pub pattern: String,
    pub replacement: Option<String>,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct AppConfig {
    pub default_provider: Option<ProviderChoice>,
    pub local_only: Option<bool>,
    pub context_lines: Option<usize>,
    pub max_context_chars: Option<usize>,
    pub yes: Option<bool>,
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub ollama_base_url: Option<String>,
    pub custom_redact_rules: Option<Vec<CustomRedactRuleConfig>>,
    pub custom_noise_patterns: Option<Vec<String>>,
    pub proxy_bind: Option<String>,
    pub upstream_url: Option<String>,
    pub proxy_token: Option<String>,
}

impl AppConfig {
    pub fn load() -> Self {
        let mut config_path = None;

        // 1. Check local project .tokenectomy.toml
        if PathBuf::from(".tokenectomy.toml").exists() {
            // VULN-05: Warn about project-level config (could be planted by malicious repo)
            eprintln!("⚠️  Loading config from local .tokenectomy.toml — verify this file is trusted.");
            config_path = Some(PathBuf::from(".tokenectomy.toml"));
        } else if let Some(mut user_dir) = dirs::home_dir() {
            // 2. Check global ~/.tokenectomy.toml
            user_dir.push(".tokenectomy.toml");
            if user_dir.exists() {
                config_path = Some(user_dir);
            }
        }

        if let Some(path) = config_path {
            if let Ok(content) = fs::read_to_string(&path) {
                match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!(
                            "⚠️  Warning: Failed to parse configuration from '{}': {}",
                            path.display(),
                            e
                        );
                    }
                }
            }
        }

        AppConfig::default()
    }
}
