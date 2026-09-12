use regex::Regex;
use std::borrow::Cow;
use std::sync::LazyLock;

struct RedactRule {
    regex: Regex,
    replacement: &'static str,
}

static REDACT_RULES: LazyLock<Vec<RedactRule>> = LazyLock::new(|| {
    vec![
        // 1. Specific Token Prefixes (run first so specific format tags are matched)
        // 1a. GitHub tokens (ghp_, gho_, ghs_, ghr_, github_pat_)
        RedactRule {
            regex: Regex::new(r"\b(ghp_|gho_|ghs_|ghr_|github_pat_)[a-zA-Z0-9_]{20,}\b").unwrap(),
            replacement: "[GITHUB_TOKEN_REDACTED]",
        },
        // 1b. Slack tokens (xoxb-, xoxp-, xoxa-, xoxr-)
        RedactRule {
            regex: Regex::new(r"\bxox[bparo]-[a-zA-Z0-9\-]{20,}\b").unwrap(),
            replacement: "[SLACK_TOKEN_REDACTED]",
        },
        // 1c. Google API keys (AIza...)
        RedactRule {
            regex: Regex::new(r"\bAIza[a-zA-Z0-9_\-]{35}\b").unwrap(),
            replacement: "[GOOGLE_API_KEY_REDACTED]",
        },
        // 1d. Anthropic API keys (must run before OpenAI because Anthropic keys start with sk-ant-)
        RedactRule {
            regex: Regex::new(r"\bsk-ant-[a-zA-Z0-9_\-]{20,}\b").unwrap(),
            replacement: "[ANTHROPIC_KEY_REDACTED]",
        },
        // 1e. OpenAI API keys
        RedactRule {
            regex: Regex::new(r"\bsk-(?:proj-)?[a-zA-Z0-9_\-]{20,}\b").unwrap(),
            replacement: "[OPENAI_KEY_REDACTED]",
        },
        // 1f. HuggingFace tokens (hf_...)
        RedactRule {
            regex: Regex::new(r"\bhf_[a-zA-Z0-9]{34,}\b").unwrap(),
            replacement: "[HUGGINGFACE_TOKEN_REDACTED]",
        },
        // 1g. npm access tokens (npm_...)
        RedactRule {
            regex: Regex::new(r"\bnpm_[a-zA-Z0-9]{30,}\b").unwrap(),
            replacement: "[NPM_TOKEN_REDACTED]",
        },
        // 1h. PyPI upload tokens (pypi-AgEI...)
        RedactRule {
            regex: Regex::new(r"\bpypi-AgEIcHlwaS5vcmc[a-zA-Z0-9_\-]{50,}\b").unwrap(),
            replacement: "[PYPI_TOKEN_REDACTED]",
        },
        // 1i. Stripe API keys (sk_live_..., rk_live_..., sk_test_...)
        RedactRule {
            regex: Regex::new(r"\b(?:sk|rk)_(?:live|test)_[a-zA-Z0-9]{24,}\b").unwrap(),
            replacement: "[STRIPE_KEY_REDACTED]",
        },
        // 1j. GitLab personal access tokens (glpat-...)
        RedactRule {
            regex: Regex::new(r"\bglpat-[a-zA-Z0-9_\-]{20,}\b").unwrap(),
            replacement: "[GITLAB_TOKEN_REDACTED]",
        },
        // 1k. SendGrid API keys (SG...)
        RedactRule {
            regex: Regex::new(r"\bSG\.[a-zA-Z0-9_\-\.]{60,}\b").unwrap(),
            replacement: "[SENDGRID_KEY_REDACTED]",
        },
        // 2. AWS Access Key ID
        RedactRule {
            regex: Regex::new(r"(?i)\b(AKIA|ASIA)[0-9A-Z]{16}\b").unwrap(),
            replacement: "[AWS_KEY_REDACTED]",
        },
        // 3. AWS Secret Access Key
        RedactRule {
            regex: Regex::new(r#"(?i)(["']?aws_secret[a-z0-9_]*["']?\s*[:=]\s*["']?)([a-zA-Z0-9/+=]{40})(["']?)"#).unwrap(),
            replacement: "${1}[AWS_SECRET_REDACTED]${3}",
        },
        // 4. JWT Tokens
        RedactRule {
            regex: Regex::new(r"eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+").unwrap(),
            replacement: "[JWT_REDACTED]",
        },
        // 5. Private keys (PEM format)
        RedactRule {
            regex: Regex::new(r"-----BEGIN[A-Z ]*PRIVATE KEY-----[\s\S]*?-----END[A-Z ]*PRIVATE KEY-----").unwrap(),
            replacement: "[PRIVATE_KEY_REDACTED]",
        },
        // 6. Connection strings (postgresql://, postgres://, mysql://, mongodb://, redis://, mssql://, amqp://)
        RedactRule {
            regex: Regex::new(r##"(?i)(postgresql|postgres|mysql|mongodb|redis|mssql|amqp)://[^\s"'<>]+"##).unwrap(),
            replacement: "[CONNECTION_STRING_REDACTED]",
        },
        // 7. Authorization header with Bearer token
        RedactRule {
            regex: Regex::new(r"(?i)Authorization:\s*Bearer\s+[a-zA-Z0-9_\-\.]+").unwrap(),
            replacement: "Authorization: Bearer [REDACTED]",
        },
        // 8. Generic Key/Token patterns (fallback for other credentials)
        // 8a. Quoted values: preserves JSON/YAML keys, colons, and quotes
        RedactRule {
            regex: Regex::new(r#"(?i)(["']?(?:api_key|token|password|secret|auth|bearer)[a-z0-9_]*["']?\s*[:=]\s*)(["'])(?:[^"'\r\n]{4,})(["'])"#).unwrap(),
            replacement: "${1}${2}[REDACTED]${3}",
        },
        // 8b. Unquoted words: preserves keys and colons/equals
        RedactRule {
            regex: Regex::new(r#"(?i)(["']?(?:api_key|token|password|secret|auth|bearer)[a-z0-9_]*["']?\s*[:=]\s*)([a-zA-Z0-9_\-\.]{10,})"#).unwrap(),
            replacement: "${1}[REDACTED]",
        },
    ]
});

pub fn redact_secrets(input: &str) -> String {
    redact_secrets_with_custom(input, &[])
}

pub fn redact_secrets_with_custom(
    input: &str,
    custom_rules: &[crate::config::CustomRedactRuleConfig],
) -> String {
    let mut redacted = input.to_string();
    for rule in REDACT_RULES.iter() {
        if let Cow::Owned(new_str) = rule.regex.replace_all(&redacted, rule.replacement) {
            redacted = new_str;
        }
    }
    for custom in custom_rules {
        if let Ok(re) = Regex::new(&custom.pattern) {
            let repl = custom
                .replacement
                .as_deref()
                .unwrap_or("[CUSTOM_SECRET_REDACTED]");
            if let Cow::Owned(new_str) = re.replace_all(&redacted, repl) {
                redacted = new_str;
            }
        }
    }
    redacted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_generic() {
        let log = "Connecting to DB with password = supersecret12345! Error.";
        let res = redact_secrets(log);
        assert_eq!(res, "Connecting to DB with password = [REDACTED]! Error.");

        let log2 = r#"{"openai_api_key": "sk-something1234567890"}"#;
        let res2 = redact_secrets(log2);
        assert!(res2.contains("[REDACTED]"));
        assert!(!res2.contains("sk-something"));
    }

    #[test]
    fn test_redact_aws() {
        let log = "export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE";
        let res = redact_secrets(log);
        assert_eq!(res, "export AWS_ACCESS_KEY_ID=[AWS_KEY_REDACTED]");
    }

    #[test]
    fn test_redact_jwt() {
        let log = "Some string eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWV9.TJVA95OrM7E2cBab30RMHrHDcEfxjoYZgeFONFh7HgQ and some other text";
        let res = redact_secrets(log);
        assert!(res.contains("[JWT_REDACTED]"));
        assert!(!res.contains("eyJhbGciOi"));
    }

    #[test]
    fn test_redact_github_token() {
        let log = "Using token ghp_ABCDEFGHIJKLMNOPQRSTuvwxyz1234";
        let res = redact_secrets(log);
        assert!(res.contains("[GITHUB_TOKEN_REDACTED]"));
        assert!(!res.contains("ghp_"));
    }

    #[test]
    fn test_redact_connection_string() {
        let log = "DATABASE_URL=postgresql://admin:secretpass@db.example.com:5432/mydb";
        let res = redact_secrets(log);
        assert!(res.contains("[CONNECTION_STRING_REDACTED]"));
        assert!(!res.contains("secretpass"));
    }

    #[test]
    fn test_redact_complex_quoted_password() {
        let log = r#"let conn = connect("postgres", password = "P@ssw0rd!#123$Secure", host = "localhost");"#;
        let res = redact_secrets(log);
        assert!(res.contains(r#"password = "[REDACTED]""#));
        assert!(!res.contains("P@ssw0rd"));
    }

    #[test]
    fn test_redact_ai_api_keys() {
        let log_openai = "Error calling API with key sk-proj-1234567890abcdef1234567890";
        let res_openai = redact_secrets(log_openai);
        assert!(res_openai.contains("[OPENAI_KEY_REDACTED]"));
        assert!(!res_openai.contains("1234567890abcdef"));

        let log_anthropic = "Anthropic auth failed: sk-ant-api03-abcdef1234567890abcdef12345";
        let res_anthropic = redact_secrets(log_anthropic);
        assert!(res_anthropic.contains("[ANTHROPIC_KEY_REDACTED]"));
        assert!(!res_anthropic.contains("abcdef1234567890"));
    }

    #[test]
    fn test_redact_preserves_valid_json() {
        let json_input = r#"{"db_host": "localhost", "db_password": "my_super_secret_12345", "api_token": "token_abc123456"}"#;
        let res = redact_secrets(json_input);
        assert!(res.contains(r#""db_password": "[REDACTED]""#));
        assert!(res.contains(r#""api_token": "[REDACTED]""#));
        // Must still parse as valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&res).expect("JSON must remain valid after redaction");
        assert_eq!(parsed["db_password"], "[REDACTED]");
        assert_eq!(parsed["api_token"], "[REDACTED]");
    }

    #[test]
    fn test_redact_developer_and_cloud_tokens() {
        let dummy_hf = format!("{}_{}", "hf", "0123456789abcdef0123456789abcdef0123");
        let log_hf = format!("Downloading model with {}", dummy_hf);
        let res_hf = redact_secrets(&log_hf);
        assert_eq!(res_hf, "Downloading model with [HUGGINGFACE_TOKEN_REDACTED]");

        let dummy_npm = format!("{}_{}", "npm", "0123456789abcdef0123456789abcdef12");
        let log_npm = format!("Publishing package with {}", dummy_npm);
        let res_npm = redact_secrets(&log_npm);
        assert_eq!(res_npm, "Publishing package with [NPM_TOKEN_REDACTED]");

        let dummy_pypi = format!("{}-{}", "pypi", "AgEIcHlwaS5vcmcCJDEyMzQ1Njc4LTBhYmMtNGFiYy05YWJjLTBhYmNkZWYwMTIzNAACTDF");
        let log_pypi = format!("Twine upload token: {}", dummy_pypi);
        let res_pypi = redact_secrets(&log_pypi);
        assert_eq!(res_pypi, "Twine upload token: [PYPI_TOKEN_REDACTED]");

        let dummy_stripe = format!("{}_{}_{}", "sk", "live", "51A2B3C4D5E6F7G8H9I0J1K2L3");
        let log_stripe = format!("Stripe webhook error with secret key {}", dummy_stripe);
        let res_stripe = redact_secrets(&log_stripe);
        assert_eq!(res_stripe, "Stripe webhook error with secret key [STRIPE_KEY_REDACTED]");

        let dummy_gitlab = format!("{}-{}", "glpat", "abcdef1234567890ABCD");
        let log_gitlab = format!("GitLab CI clone error: {}", dummy_gitlab);
        let res_gitlab = redact_secrets(&log_gitlab);
        assert_eq!(res_gitlab, "GitLab CI clone error: [GITLAB_TOKEN_REDACTED]");

        let dummy_sendgrid = format!("{}.{}", "SG", "abcdefghijklmnopqrstuvwxyz0123456789012345678901234567890123456789");
        let log_sendgrid = format!("Email delivery failed with api key {}", dummy_sendgrid);
        let res_sendgrid = redact_secrets(&log_sendgrid);
        assert_eq!(res_sendgrid, "Email delivery failed with api key [SENDGRID_KEY_REDACTED]");
    }

    #[test]
    fn test_redact_with_custom_rules() {
        let custom_rules = vec![
            crate::config::CustomRedactRuleConfig {
                pattern: r"ACME-[0-9]{5}-[A-Z]+".to_string(),
                replacement: Some("[ACME_LICENSE_REDACTED]".to_string()),
            },
            crate::config::CustomRedactRuleConfig {
                pattern: r"internal_secret_[a-z0-9]+".to_string(),
                replacement: None, // Should fallback to [CUSTOM_SECRET_REDACTED]
            },
        ];

        let log = "Connecting ACME-12345-PROD with token internal_secret_9988aabb and key sk-ant-api03-abcdef1234567890abcdef12345";
        let res = redact_secrets_with_custom(log, &custom_rules);

        assert!(res.contains("[ACME_LICENSE_REDACTED]"));
        assert!(res.contains("[CUSTOM_SECRET_REDACTED]"));
        assert!(res.contains("[ANTHROPIC_KEY_REDACTED]"));
        assert!(!res.contains("ACME-12345-PROD"));
        assert!(!res.contains("internal_secret_9988aabb"));
    }
}
