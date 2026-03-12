// ContextPaste — Content Type Classifier
// Detects 15 content types using regex patterns.
// Priority: Credential > specific types > PlainText

use lazy_static::lazy_static;
use regex::Regex;

use crate::clipboard::credential_detector;
use crate::storage::models::ContentType;

lazy_static! {
    // URL
    static ref RE_URL: Regex = Regex::new(r"(?i)^(https?://|www\.)").unwrap();

    // Email
    static ref RE_EMAIL: Regex = Regex::new(
        r"^[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}$"
    ).unwrap();

    // IP Address (IPv4 or IPv6)
    static ref RE_IPV4: Regex = Regex::new(
        r"^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}(/\d{1,2})?$"
    ).unwrap();
    static ref RE_IPV6: Regex = Regex::new(
        r"(?i)^([0-9a-f]{0,4}:){2,7}[0-9a-f]{0,4}$"
    ).unwrap();

    // SQL
    static ref RE_SQL: Regex = Regex::new(
        r"(?i)^\s*(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER|DROP)\s"
    ).unwrap();

    // Shell commands
    static ref RE_SHELL: Regex = Regex::new(
        r"(?i)^\s*(kubectl|docker|aws|git|npm|pnpm|yarn|curl|wget|ssh|cd|ls|grep|sudo|chmod|chown|cat|echo|export|pip|cargo|rustup)\s"
    ).unwrap();

    // Code patterns
    static ref RE_CODE: Regex = Regex::new(
        r"(?m)(^|\s)(function\s|def\s|class\s|import\s|const\s|let\s|var\s|=>|->|pub\s+fn|async\s+fn|interface\s|type\s+\w+\s*=|enum\s)"
    ).unwrap();

    // AWS ARN
    static ref RE_ARN: Regex = Regex::new(r"^arn:aws:").unwrap();

    // Connection strings
    static ref RE_CONN: Regex = Regex::new(
        r"(?i)^(postgres|postgresql|mysql|mongodb|redis|amqp|sqlite)://"
    ).unwrap();

    // File path
    static ref RE_FILEPATH_UNIX: Regex = Regex::new(r"^[~/][\w.\-/]+").unwrap();
    static ref RE_FILEPATH_WIN: Regex = Regex::new(r"(?i)^[A-Z]:\\[\w.\-\\]+").unwrap();

    // Markdown
    static ref RE_MARKDOWN: Regex = Regex::new(
        r"(^#{1,6}\s|\*\*[^*]+\*\*|- \[ \]|\[[^\]]+\]\([^)]+\))"
    ).unwrap();

    // HTML/XML
    static ref RE_HTML: Regex = Regex::new(r"^\s*<[a-zA-Z][\s\S]*</[a-zA-Z]").unwrap();

    // YAML (multi-line key: value or starts with ---)
    static ref RE_YAML: Regex = Regex::new(
        r"(?m)(^---\s*$|^[a-zA-Z_][\w]*:\s+\S)"
    ).unwrap();
}

/// Classify content into one of 15 content types.
/// Credential detection has highest priority.
pub fn classify(content: &str) -> ContentType {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return ContentType::PlainText;
    }

    // Highest priority: Credential
    if credential_detector::detect(content).is_some() {
        return ContentType::Credential;
    }

    // AWS ARN
    if RE_ARN.is_match(trimmed) {
        return ContentType::AwsArn;
    }

    // Connection string
    if RE_CONN.is_match(trimmed) {
        return ContentType::ConnectionString;
    }

    // URL (check before email since emails contain @)
    if RE_URL.is_match(trimmed) {
        return ContentType::Url;
    }

    // Email (single line only)
    if !trimmed.contains('\n') && RE_EMAIL.is_match(trimmed) {
        return ContentType::Email;
    }

    // IP address
    if RE_IPV4.is_match(trimmed) || RE_IPV6.is_match(trimmed) {
        return ContentType::IpAddress;
    }

    // JSON (starts with { or [ and parses)
    if (trimmed.starts_with('{') || trimmed.starts_with('['))
        && serde_json::from_str::<serde_json::Value>(trimmed).is_ok()
    {
        return ContentType::Json;
    }

    // SQL
    if RE_SQL.is_match(trimmed) {
        return ContentType::Sql;
    }

    // Shell command
    if RE_SHELL.is_match(trimmed) {
        return ContentType::ShellCommand;
    }

    // HTML/XML
    if RE_HTML.is_match(trimmed) {
        return ContentType::HtmlXml;
    }

    // YAML (needs multiple indicators or starts with ---)
    if RE_YAML.is_match(trimmed) && trimmed.contains('\n') {
        return ContentType::Yaml;
    }

    // Code
    if RE_CODE.is_match(trimmed) {
        return ContentType::Code;
    }

    // Markdown
    if RE_MARKDOWN.is_match(trimmed) {
        return ContentType::Markdown;
    }

    // File path
    if !trimmed.contains('\n')
        && (RE_FILEPATH_UNIX.is_match(trimmed) || RE_FILEPATH_WIN.is_match(trimmed))
    {
        return ContentType::FilePath;
    }

    ContentType::PlainText
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url() {
        assert_eq!(classify("https://example.com"), ContentType::Url);
        assert_eq!(classify("http://localhost:3000"), ContentType::Url);
        assert_eq!(classify("www.google.com"), ContentType::Url);
    }

    #[test]
    fn test_email() {
        assert_eq!(classify("user@example.com"), ContentType::Email);
        assert_eq!(classify("test.name+tag@domain.co.uk"), ContentType::Email);
    }

    #[test]
    fn test_ip() {
        assert_eq!(classify("192.168.1.1"), ContentType::IpAddress);
        assert_eq!(classify("10.0.0.0/8"), ContentType::IpAddress);
        assert_eq!(classify("::1"), ContentType::IpAddress);
    }

    #[test]
    fn test_json() {
        assert_eq!(classify(r#"{"key": "value"}"#), ContentType::Json);
        assert_eq!(classify(r#"[1, 2, 3]"#), ContentType::Json);
        // Invalid JSON should not match
        assert_ne!(classify("{not json"), ContentType::Json);
    }

    #[test]
    fn test_sql() {
        assert_eq!(classify("SELECT * FROM users"), ContentType::Sql);
        assert_eq!(
            classify("INSERT INTO table VALUES (1)"),
            ContentType::Sql
        );
        assert_eq!(classify("  DELETE FROM logs"), ContentType::Sql);
        assert_eq!(classify("CREATE TABLE foo (id INT)"), ContentType::Sql);
        assert_eq!(classify("update users set name='test'"), ContentType::Sql);
    }

    #[test]
    fn test_shell() {
        assert_eq!(classify("git status"), ContentType::ShellCommand);
        assert_eq!(classify("docker ps -a"), ContentType::ShellCommand);
        assert_eq!(classify("npm install react"), ContentType::ShellCommand);
        assert_eq!(classify("curl https://api.com"), ContentType::ShellCommand);
        assert_eq!(classify("kubectl get pods"), ContentType::ShellCommand);
        assert_eq!(classify("cargo build"), ContentType::ShellCommand);
    }

    #[test]
    fn test_code() {
        assert_eq!(
            classify("function hello() { return 1; }"),
            ContentType::Code
        );
        assert_eq!(classify("const x = 42;"), ContentType::Code);
        assert_eq!(classify("import os"), ContentType::Code);
        assert_eq!(classify("pub fn main() {}"), ContentType::Code);
        assert_eq!(classify("class MyClass:"), ContentType::Code);
        assert_eq!(classify("def foo():"), ContentType::Code);
    }

    #[test]
    fn test_aws_arn() {
        assert_eq!(
            classify("arn:aws:s3:::my-bucket"),
            ContentType::AwsArn
        );
        assert_eq!(
            classify("arn:aws:iam::123456:role/MyRole"),
            ContentType::AwsArn
        );
    }

    #[test]
    fn test_connection_string() {
        // Connection strings with passwords are classified as Credential (higher priority)
        assert_eq!(
            classify("postgres://user:pass@localhost/db"),
            ContentType::Credential
        );
        // Connection strings without passwords are classified as ConnectionString
        assert_eq!(
            classify("redis://localhost:6379"),
            ContentType::ConnectionString
        );
        // Connection strings with passwords → Credential
        assert_eq!(
            classify("mongodb://user:pass@host:27017/db"),
            ContentType::Credential
        );
    }

    #[test]
    fn test_filepath() {
        assert_eq!(classify("/usr/local/bin/node"), ContentType::FilePath);
        assert_eq!(classify("~/projects/app"), ContentType::FilePath);
        assert_eq!(classify("C:\\Users\\test\\file.txt"), ContentType::FilePath);
    }

    #[test]
    fn test_markdown() {
        assert_eq!(classify("# Hello World"), ContentType::Markdown);
        assert_eq!(
            classify("Click **here** for more"),
            ContentType::Markdown
        );
        assert_eq!(
            classify("[link](https://example.com)"),
            ContentType::Markdown
        );
    }

    #[test]
    fn test_html() {
        assert_eq!(
            classify("<div>Hello</div>"),
            ContentType::HtmlXml
        );
        assert_eq!(
            classify("<html><body>test</body></html>"),
            ContentType::HtmlXml
        );
    }

    #[test]
    fn test_yaml() {
        assert_eq!(
            classify("---\nname: test\nvalue: 42"),
            ContentType::Yaml
        );
        assert_eq!(
            classify("apiVersion: v1\nkind: Pod"),
            ContentType::Yaml
        );
    }

    #[test]
    fn test_plain_text() {
        assert_eq!(classify("just some text"), ContentType::PlainText);
        assert_eq!(classify("hello world"), ContentType::PlainText);
        assert_eq!(classify(""), ContentType::PlainText);
    }
}
