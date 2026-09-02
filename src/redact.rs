use regex::Regex;

pub fn redact_secrets(input: &str) -> String {
    let mut redacted = input.to_string();

    // 1. Generic Key/Token patterns (env vars, JSON, etc)
    let re_generic = Regex::new(r#"(?i)(api_key|token|password|secret|auth|bearer)[\s:="']+([a-zA-Z0-9_\-\.]{10,})"#).unwrap();
    redacted = re_generic.replace_all(&redacted, "$1 = [REDACTED]").to_string();

    // 2. AWS Access Key ID
    let re_aws_key = Regex::new(r"(?i)\b(AKIA|ASIA)[0-9A-Z]{16}\b").unwrap();
    redacted = re_aws_key.replace_all(&redacted, "[AWS_KEY_REDACTED]").to_string();

    // 3. AWS Secret Access Key
    let re_aws_secret = Regex::new(r#"(?i)(aws_secret[a-z_]*)[=:\s"']+([a-zA-Z0-9/+=]{40})"#).unwrap();
    redacted = re_aws_secret.replace_all(&redacted, "$1 = [AWS_SECRET_REDACTED]").to_string();

    // 4. JWT Tokens
    let re_jwt = Regex::new(r"eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+").unwrap();
    redacted = re_jwt.replace_all(&redacted, "[JWT_REDACTED]").to_string();

    // VULN-06: Additional secret patterns

    // 5. GitHub tokens (ghp_, gho_, ghs_, ghr_, github_pat_)
    let re_github = Regex::new(r"\b(ghp_|gho_|ghs_|ghr_|github_pat_)[a-zA-Z0-9_]{20,}\b").unwrap();
    redacted = re_github.replace_all(&redacted, "[GITHUB_TOKEN_REDACTED]").to_string();

    // 6. Slack tokens (xoxb-, xoxp-, xoxa-, xoxr-)
    let re_slack = Regex::new(r"\bxox[bparo]-[a-zA-Z0-9\-]{20,}\b").unwrap();
    redacted = re_slack.replace_all(&redacted, "[SLACK_TOKEN_REDACTED]").to_string();

    // 7. Google API keys (AIza...)
    let re_google = Regex::new(r"\bAIza[a-zA-Z0-9_\-]{35}\b").unwrap();
    redacted = re_google.replace_all(&redacted, "[GOOGLE_API_KEY_REDACTED]").to_string();

    // 8. Private keys (PEM format)
    let re_pem = Regex::new(r"-----BEGIN[A-Z ]*PRIVATE KEY-----[\s\S]*?-----END[A-Z ]*PRIVATE KEY-----").unwrap();
    redacted = re_pem.replace_all(&redacted, "[PRIVATE_KEY_REDACTED]").to_string();

    // 9. Connection strings (postgresql://, mysql://, mongodb://, redis://)
    let re_connstr = Regex::new(r"(?i)(postgresql|mysql|mongodb|redis|amqp)://[^\s]+").unwrap();
    redacted = re_connstr.replace_all(&redacted, "[CONNECTION_STRING_REDACTED]").to_string();

    // 10. Authorization header with Bearer token
    let re_bearer = Regex::new(r"(?i)Authorization:\s*Bearer\s+[a-zA-Z0-9_\-\.]+").unwrap();
    redacted = re_bearer.replace_all(&redacted, "Authorization: Bearer [REDACTED]").to_string();

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
}
