// ContextPaste — Data Models
// Must stay in sync with TypeScript types in src/lib/types.ts

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    Url,
    Email,
    IpAddress,
    Json,
    Yaml,
    Sql,
    ShellCommand,
    Code,
    AwsArn,
    ConnectionString,
    FilePath,
    Credential,
    Markdown,
    HtmlXml,
    PlainText,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Url => "Url",
            Self::Email => "Email",
            Self::IpAddress => "IpAddress",
            Self::Json => "Json",
            Self::Yaml => "Yaml",
            Self::Sql => "Sql",
            Self::ShellCommand => "ShellCommand",
            Self::Code => "Code",
            Self::AwsArn => "AwsArn",
            Self::ConnectionString => "ConnectionString",
            Self::FilePath => "FilePath",
            Self::Credential => "Credential",
            Self::Markdown => "Markdown",
            Self::HtmlXml => "HtmlXml",
            Self::PlainText => "PlainText",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Url" => Self::Url,
            "Email" => Self::Email,
            "IpAddress" => Self::IpAddress,
            "Json" => Self::Json,
            "Yaml" => Self::Yaml,
            "Sql" => Self::Sql,
            "ShellCommand" => Self::ShellCommand,
            "Code" => Self::Code,
            "AwsArn" => Self::AwsArn,
            "ConnectionString" => Self::ConnectionString,
            "FilePath" => Self::FilePath,
            "Credential" => Self::Credential,
            "Markdown" => Self::Markdown,
            "HtmlXml" => Self::HtmlXml,
            _ => Self::PlainText,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipItem {
    pub id: String,
    pub content: String,
    pub content_type: ContentType,
    pub content_hash: String,
    pub content_length: i64,
    pub is_credential: bool,
    pub credential_type: Option<String>,
    pub source_app: Option<String>,
    pub source_window_title: Option<String>,
    pub is_pinned: bool,
    pub is_starred: bool,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub last_pasted_at: Option<String>,
    pub paste_count: i64,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankedItem {
    pub item: ClipItem,
    pub score: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasteEvent {
    pub id: String,
    pub item_id: String,
    pub target_app: Option<String>,
    pub target_window_title: Option<String>,
    pub pasted_at: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictionStat {
    pub id: i64,
    pub content_type: String,
    pub source_app: Option<String>,
    pub target_app: String,
    pub frequency: i64,
    pub last_used_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowChain {
    pub id: String,
    pub chain_hash: String,
    pub items_json: String,
    pub frequency: i64,
    pub last_triggered_at: String,
    pub source_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainItem {
    pub content_type: ContentType,
    pub position: i32,
    pub preview: String,
}
