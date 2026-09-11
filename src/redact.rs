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
    let mut redacted = input.to_string();
    for rule in REDACT_RULES.iter() {
        if let Cow::Owned(new_str) = rule.regex.replace_all(&redacted, rule.replacement) {
            redacted = new_str;
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
}
