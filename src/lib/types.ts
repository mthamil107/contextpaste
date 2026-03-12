// ContextPaste — TypeScript Types
// This file is the single source of truth for all frontend types.
// Must stay in sync with Rust structs in src-tauri/src/storage/models.rs

export type ContentType =
  | "Url"
  | "Email"
  | "IpAddress"
  | "Json"
  | "Yaml"
  | "Sql"
  | "ShellCommand"
  | "Code"
  | "AwsArn"
  | "ConnectionString"
  | "FilePath"
  | "Credential"
  | "Markdown"
  | "HtmlXml"
  | "PlainText";

export interface ClipItem {
  id: string;
  content: string;
  contentType: ContentType;
  contentLength: number;
  isCredential: boolean;
  credentialType?: string;
  sourceApp?: string;
  sourceWindowTitle?: string;
  isPinned: boolean;
  isStarred: boolean;
  expiresAt?: string;
  createdAt: string;
  lastPastedAt?: string;
  pasteCount: number;
  tags?: string[];
}

export interface RankedItem {
  item: ClipItem;
  score: number;
  reason: "frequency" | "recency" | "type_match" | "chain" | "pinned";
}

export interface WorkflowChain {
  id: string;
  items: ChainItem[];
  frequency: number;
  lastTriggeredAt: string;
}

export interface ChainItem {
  contentType: ContentType;
  position: number;
  preview: string;
}

export interface PasteContext {
  targetApp: string;
  targetWindowTitle: string;
  timestamp: string;
}

export type AIProvider = "local" | "openai" | "anthropic" | "ollama";

export interface AppSettings {
  maxHistoryItems: number;
  credentialAutoExpireMinutes: number;
  hotkeyQuickPaste: string;
  hotkeyHistory: string;
  theme: "system" | "light" | "dark";
  showSourceContext: boolean;
  showTypeBadges: boolean;
  enablePredictions: boolean;
  enableWorkflowChains: boolean;
  enableSemanticSearch: boolean;
  aiProvider: AIProvider;
  ignoredApps: string[];
  startupOnLogin: boolean;
  overlayPosition: "cursor" | "center" | "top-right";
  overlayMaxItems: number;
  dedupEnabled: boolean;
  dedupWindowSeconds: number;
}

// Content type display configuration
export const CONTENT_TYPE_CONFIG: Record<
  ContentType,
  { label: string; color: string; icon: string }
> = {
  Url: { label: "URL", color: "badge-url", icon: "link" },
  Email: { label: "Email", color: "badge-email", icon: "mail" },
  IpAddress: { label: "IP", color: "badge-ip", icon: "globe" },
  Json: { label: "JSON", color: "badge-json", icon: "braces" },
  Yaml: { label: "YAML", color: "badge-yaml", icon: "file-code" },
  Sql: { label: "SQL", color: "badge-sql", icon: "database" },
  ShellCommand: { label: "Shell", color: "badge-shell", icon: "terminal" },
  Code: { label: "Code", color: "badge-code", icon: "code" },
  AwsArn: { label: "AWS", color: "badge-aws", icon: "cloud" },
  ConnectionString: { label: "Conn", color: "badge-connection", icon: "plug" },
  FilePath: { label: "Path", color: "badge-filepath", icon: "folder" },
  Credential: { label: "Secret", color: "badge-credential", icon: "lock" },
  Markdown: { label: "MD", color: "badge-markdown", icon: "file-text" },
  HtmlXml: { label: "HTML", color: "badge-html", icon: "code-xml" },
  PlainText: { label: "Text", color: "badge-plain", icon: "type" },
};
