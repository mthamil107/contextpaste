// ContextPaste — Credential Detector
// Detects secrets, API keys, tokens, private keys, and passwords.
// Masking: show first 4 + last 4 chars, replace middle with ••••••••

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialMatch {
    pub credential_type: String,
    pub matched_text: String,
}

lazy_static! {
    static ref PATTERNS: Vec<(&'static str, Regex)> = vec![
        (
            "AWS Access Key",
            Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
        ),
        (
            "GitHub Token",
            Regex::new(r"gh[ps]_[A-Za-z0-9_]{36,}").unwrap(),
        ),
        (
            "GitLab Token",
            Regex::new(r"glpat-[A-Za-z0-9\-]{20,}").unwrap(),
        ),
        (
            "Anthropic API Key",
            Regex::new(r"sk-ant-[A-Za-z0-9\-_]{40,}").unwrap(),
        ),
        (
            "OpenAI API Key",
            Regex::new(r"sk-[A-Za-z0-9]{48,}").unwrap(),
        ),
        (
            "JWT Token",
            Regex::new(r"eyJ[A-Za-z0-9\-_]+\.eyJ[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+").unwrap(),
        ),
        (
            "Private Key",
            Regex::new(r"-----BEGIN (RSA |EC )?PRIVATE KEY-----").unwrap(),
        ),
        (
            "Connection with Password",
            Regex::new(r"://[^:]+:[^@]+@").unwrap(),
        ),
        (
            "Generic API Key",
            Regex::new(
                r#"(?i)(api[_-]?key|apikey|token|secret|password|passwd|authorization)\s*[:=]\s*['"]?[A-Za-z0-9\-_./+]{32,}"#
            ).unwrap(),
        ),
        (
            "AWS Secret Key",
            Regex::new(
                r#"(?i)(aws_secret_access_key|aws_secret)\s*[:=]\s*['"]?[A-Za-z0-9/+=]{40}"#
            ).unwrap(),
        ),
    ];
}

/// Detect if content contains a credential. Returns the first match found.
pub fn detect(content: &str) -> Option<CredentialMatch> {
    for (cred_type, regex) in PATTERNS.iter() {
        if let Some(m) = regex.find(content) {
            return Some(CredentialMatch {
                credential_type: cred_type.to_string(),
                matched_text: m.as_str().to_string(),
            });
        }
    }
    None
}

/// Detect all credentials in content.
pub fn detect_all(content: &str) -> Vec<CredentialMatch> {
    let mut matches = Vec::new();
    for (cred_type, regex) in PATTERNS.iter() {
        for m in regex.find_iter(content) {
            matches.push(CredentialMatch {
                credential_type: cred_type.to_string(),
                matched_text: m.as_str().to_string(),
            });
        }
    }
    matches
}

/// Mask a credential: show first 4 + last 4 chars, replace middle with ••••••••
pub fn mask(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= 8 {
        return "••••••••".to_string();
    }
    let first: String = chars[..4].iter().collect();
    let last: String = chars[chars.len() - 4..].iter().collect();
    format!("{}••••••••{}", first, last)
}

/// Mask all detected credentials in content, returning the masked version.
pub fn mask_content(content: &str) -> String {
    let mut result = content.to_string();
    for (_, regex) in PATTERNS.iter() {
        let replaced = regex
            .replace_all(&result, |caps: &regex::Captures| mask(&caps[0]))
            .to_string();
        result = replaced;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_access_key() {
        let m = detect("AKIAIOSFODNN7EXAMPLE").unwrap();
        assert_eq!(m.credential_type, "AWS Access Key");
    }

    #[test]
    fn test_github_token() {
        let token = format!("ghp_{}", "a".repeat(36));
        let m = detect(&token).unwrap();
        assert_eq!(m.credential_type, "GitHub Token");
    }

    #[test]
    fn test_gitlab_token() {
        let token = format!("glpat-{}", "a".repeat(20));
        let m = detect(&token).unwrap();
        assert_eq!(m.credential_type, "GitLab Token");
    }

    #[test]
    fn test_anthropic_key() {
        let key = format!("sk-ant-{}", "a".repeat(40));
        let m = detect(&key).unwrap();
        assert_eq!(m.credential_type, "Anthropic API Key");
    }

    #[test]
    fn test_openai_key() {
        let key = format!("sk-{}", "a".repeat(48));
        let m = detect(&key).unwrap();
        assert_eq!(m.credential_type, "OpenAI API Key");
    }

    #[test]
    fn test_jwt() {
        let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.abc123def456";
        let m = detect(jwt).unwrap();
        assert_eq!(m.credential_type, "JWT Token");
    }

    #[test]
    fn test_private_key() {
        let m = detect("-----BEGIN RSA PRIVATE KEY-----").unwrap();
        assert_eq!(m.credential_type, "Private Key");

        let m2 = detect("-----BEGIN PRIVATE KEY-----").unwrap();
        assert_eq!(m2.credential_type, "Private Key");
    }

    #[test]
    fn test_connection_with_password() {
        let m = detect("postgres://user:secret@localhost/db").unwrap();
        assert_eq!(m.credential_type, "Connection with Password");
    }

    #[test]
    fn test_generic_api_key() {
        let key = format!("api_key: {}", "a".repeat(32));
        let m = detect(&key).unwrap();
        assert_eq!(m.credential_type, "Generic API Key");
    }

    #[test]
    fn test_no_credential() {
        assert!(detect("just normal text").is_none());
        assert!(detect("https://example.com").is_none());
        assert!(detect("192.168.1.1").is_none());
    }

    #[test]
    fn test_mask() {
        assert_eq!(mask("AKIAIOSFODNN7EXAMPLE"), "AKIA••••••••MPLE");
        assert_eq!(mask("short"), "••••••••");
    }

    #[test]
    fn test_mask_content() {
        let content = "my key is AKIAIOSFODNN7EXAMPLE ok";
        let masked = mask_content(content);
        assert!(masked.contains("AKIA••••••••MPLE"));
        assert!(!masked.contains("AKIAIOSFODNN7EXAMPLE"));
    }
}
