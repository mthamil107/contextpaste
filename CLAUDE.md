# ContextPaste — CLAUDE.md

> **AI-Powered Smart Clipboard Manager**
> License: GPL v3 | Fully Free | Cross-Platform (Windows, macOS, Linux)

## Project Overview

ContextPaste is a fully free, open-source, cross-platform clipboard manager that uses local AI to predict what you want to paste based on learned workflow patterns. Built with Tauri 2.x (Rust) + React 18 (TypeScript) + Tailwind CSS.

**Core Innovation**: No other clipboard manager learns WHERE you paste WHAT. ContextPaste tracks copy-paste patterns across apps, detects workflow chains, and proactively suggests the right clipboard item for the right context.

**Distribution**: Free exe/msi/dmg/deb downloads. Local AI (ONNX) ships with app. Optional BYOK (Bring Your Own Key) for cloud AI providers.

---

## Architecture

```
TAURI SHELL (Rust)
├── Clipboard Monitor (OS-native via tauri-plugin-clipboard)
├── Global Shortcut Handler (tauri-plugin-global-shortcut)
├── System Tray Manager (tauri::tray)
│
├── RUST BACKEND CORE
│   ├── Classifier Engine (regex-based content type detection)
│   ├── Credential Detector (pattern matching for secrets/keys)
│   ├── Predictor Engine (frequency + recency scoring)
│   ├── Workflow Tracker (sequential copy-paste chain detection)
│   ├── SQLite Store (rusqlite, WAL mode, FTS5)
│   ├── Vector Store (sqlite-vec for semantic search)
│   └── AI Module (ONNX Runtime for local embeddings)
│
└── REACT FRONTEND (WebView via IPC)
    ├── Quick Paste Overlay (primary UI — Ctrl+Shift+V)
    ├── History Browser (full window — Ctrl+Shift+H)
    └── Settings Panel (all configuration)
```

### Data Flow — COPY

```
OS Clipboard Change → Rust Listener
  ├── [sync, <1ms]  → Content Type Classification (regex)
  ├── [sync, <1ms]  → Credential Detection (regex)
  ├── [sync, <1ms]  → Source Context Capture (active window)
  ├── [sync]         → In-Memory Hot Cache (last 20 items)
  ├── [async]        → SQLite Persistence
  ├── [async]        → Vector Embedding (ONNX, ~20ms)
  └── [async]        → Workflow Pattern Update
```

### Data Flow — PASTE

```
Hotkey Pressed → Get Active Window Context
  → Prediction Engine Ranks Items (<10ms from hot cache)
  → Quick Paste Overlay Shown
  → User Selects (Enter/Arrow+Enter)
  → Write to System Clipboard → Simulate Ctrl+V → Dismiss Overlay
```

---

## Critical Rules — MUST FOLLOW

### Rust Backend Rules
1. **NEVER use `.unwrap()` in production code** — use `?` operator, `.unwrap_or_default()`, or proper error handling
2. **All Tauri commands return `Result<T, String>`** — frontend expects error strings
3. **Clipboard operations are the SYNC hot path** — NEVER block with async AI/DB work on copy capture
4. **Paste latency target: <50ms** — pre-cache top 20 items in memory, lazy-load AI models
5. **Credentials are NEVER included in**: embeddings, vector search index, workflow analytics, or logs
6. **Database: WAL mode always** — enable on connection, use single connection pool via `Arc<Mutex<Connection>>`
7. **All new SQLite tables need migrations** — add to `database.rs` migration array
8. **Use `thiserror` for custom error types** — not string errors in internal code
9. **Log with `log` crate** — debug for dev, info for operations, warn for recoverable, error for failures

### Frontend Rules
1. **No `any` type** — everything must be explicitly typed
2. **All Tauri IPC calls go through `src/lib/tauri.ts` wrappers** — never call `invoke()` directly from components
3. **Zustand stores are the single source of truth** — components read from stores, not local state for shared data
4. **Use `cmdk` pattern for Quick Paste overlay** — command palette UX with keyboard navigation
5. **Tailwind only for styling** — no inline styles, no CSS modules (except overlay.css for animations)
6. **Dark mode via `class` strategy** — `dark:` prefix classes, toggle via data attribute on `<html>`
7. **All text content must handle overflow** — truncate with ellipsis, never break layout

### IPC Contract Rules
1. **Rust struct field names**: `snake_case` — Tauri auto-converts to `camelCase` for frontend
2. **Dates**: ISO 8601 strings (`datetime('now')` in SQLite, `toISOString()` in JS)
3. **IDs**: UUID v4 strings
4. **Events from Rust**: use `app_handle.emit("event-name", payload)`
5. **Events in Frontend**: use `listen("event-name", callback)` from `@tauri-apps/api/event`

---

## Technology Stack

### Rust Dependencies (src-tauri/Cargo.toml)

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-clipboard = "2"
tauri-plugin-global-shortcut = "2"
tauri-plugin-fs = "2"
tauri-plugin-notification = "2"
tauri-plugin-store = "2"
tauri-plugin-os = "2"
tauri-plugin-single-instance = "2"
tauri-plugin-updater = "2"
tauri-plugin-window-state = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled", "vtab"] }
tokio = { version = "1", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
regex = "1"
lazy_static = "1"
log = "0.4"
env_logger = "0.11"
thiserror = "1"
sha2 = "0.10"
active-win-pos-rs = "0.8"
```

### Frontend Dependencies (package.json)

```json
{
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-global-shortcut": "^2.0.0",
    "@tauri-apps/plugin-os": "^2.0.0",
    "@tauri-apps/plugin-store": "^2.0.0",
    "react": "^18.3.0",
    "react-dom": "^18.3.0",
    "zustand": "^4.5.0",
    "cmdk": "^1.0.0",
    "lucide-react": "^0.400.0",
    "clsx": "^2.1.0",
    "date-fns": "^3.6.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@types/react": "^18.3.0",
    "@types/react-dom": "^18.3.0",
    "typescript": "^5.4.0",
    "vite": "^5.0.0",
    "@vitejs/plugin-react": "^4.3.0",
    "tailwindcss": "^3.4.0",
    "autoprefixer": "^10.0.0",
    "postcss": "^8.0.0",
    "vitest": "^1.0.0",
    "@testing-library/react": "^15.0.0"
  }
}
```

---

## Project Structure

```
contextpaste/
├── CLAUDE.md                          ← YOU ARE HERE
├── LICENSE                            # GPL v3
├── README.md
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js
├── postcss.config.js
├── index.html
│
├── src-tauri/                         # RUST BACKEND
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/
│   └── src/
│       ├── main.rs                    # Entry point
│       ├── lib.rs                     # Tauri setup, command registration
│       ├── commands/                  # Tauri IPC command handlers
│       │   ├── mod.rs
│       │   ├── clipboard.rs           # get_recent_items, delete_item, paste_item, etc.
│       │   ├── search.rs              # search_items, semantic_search
│       │   ├── settings.rs            # get_all_settings, update_setting, etc.
│       │   └── prediction.rs          # get_predictions
│       ├── clipboard/                 # Clipboard core logic
│       │   ├── mod.rs
│       │   ├── monitor.rs             # OS clipboard change listener
│       │   ├── classifier.rs          # Content type classification (15 types)
│       │   └── credential_detector.rs # Secret/key pattern detection
│       ├── storage/                   # Database layer
│       │   ├── mod.rs
│       │   ├── database.rs            # SQLite setup, migrations, connection pool
│       │   ├── models.rs              # Rust structs (ClipItem, PasteEvent, etc.)
│       │   └── queries.rs             # All SQL queries as functions
│       ├── prediction/                # Paste prediction engine
│       │   ├── mod.rs
│       │   ├── engine.rs              # Ranking/scoring logic
│       │   ├── workflow.rs            # Chain detection
│       │   └── context.rs             # Active window context capture
│       ├── ai/                        # AI module (Phase 3)
│       │   ├── mod.rs
│       │   ├── embeddings.rs          # ONNX embedding generation
│       │   ├── semantic_search.rs     # Vector similarity search
│       │   └── api_client.rs          # BYOK API client
│       ├── tray/                      # System tray
│       │   ├── mod.rs
│       │   └── menu.rs
│       └── utils/
│           ├── mod.rs
│           └── config.rs              # App configuration helpers
│
├── src/                               # REACT FRONTEND
│   ├── main.tsx                       # Entry point
│   ├── App.tsx                        # Root component
│   ├── index.css                      # Tailwind imports + custom styles
│   ├── components/
│   │   ├── QuickPaste/                # PRIMARY UI — overlay popup
│   │   │   ├── QuickPasteOverlay.tsx  # Container with cmdk
│   │   │   ├── ClipItem.tsx           # Single item row
│   │   │   ├── SearchBar.tsx          # Inline search
│   │   │   ├── TypeBadge.tsx          # Content type colored badge
│   │   │   └── PredictionIndicator.tsx
│   │   ├── History/                   # Full history browser
│   │   │   ├── HistoryPanel.tsx
│   │   │   ├── FilterBar.tsx
│   │   │   └── ClipDetail.tsx
│   │   ├── Settings/                  # Settings panel
│   │   │   ├── SettingsPanel.tsx
│   │   │   ├── GeneralSettings.tsx
│   │   │   ├── ShortcutSettings.tsx
│   │   │   ├── SecuritySettings.tsx
│   │   │   └── AISettings.tsx
│   │   └── shared/
│   │       ├── Kbd.tsx
│   │       ├── Tooltip.tsx
│   │       └── ThemeToggle.tsx
│   ├── hooks/
│   │   ├── useClipboard.ts
│   │   ├── useSearch.ts
│   │   ├── useShortcut.ts
│   │   └── useSettings.ts
│   ├── stores/
│   │   ├── clipboardStore.ts
│   │   ├── settingsStore.ts
│   │   └── uiStore.ts
│   └── lib/
│       ├── tauri.ts                   # IPC wrapper functions
│       ├── types.ts                   # ALL TypeScript types
│       └── constants.ts               # App constants
│
├── tests/
│   ├── rust/
│   └── e2e/
│
├── scripts/
│   ├── download-model.sh
│   └── setup-dev.sh
│
└── .github/
    └── workflows/
        ├── ci.yml
        └── release.yml
```

---

## Content Type Classification

The classifier in `src-tauri/src/clipboard/classifier.rs` must detect these 15 types:

| ContentType | Detection Logic |
|-------------|----------------|
| `Url` | Starts with `http://`, `https://`, or `www.` |
| `Email` | Matches standard email regex |
| `IpAddress` | Matches IPv4 `\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}` or IPv6 |
| `Json` | Starts with `{` or `[` AND valid JSON parse succeeds |
| `Yaml` | Contains `---` or multi-line `key: value` pattern |
| `Sql` | Starts with SELECT, INSERT, UPDATE, DELETE, CREATE, ALTER, DROP (case-insensitive) |
| `ShellCommand` | Starts with: kubectl, docker, aws, git, npm, pnpm, yarn, curl, wget, ssh, cd, ls, grep, sudo, chmod, chown, cat, echo, export, pip, cargo, rustup |
| `Code` | Contains: function, def, class, import, const, let, var, =>, ->, pub fn, async fn, interface, type, enum |
| `AwsArn` | Starts with `arn:aws:` |
| `ConnectionString` | Contains `://` with protocol: postgres, mysql, mongodb, redis, amqp, sqlite |
| `FilePath` | Matches `/path/to/file` or `C:\path\to\file` or `~/path` |
| `Credential` | Detected by credential_detector (see below) |
| `Markdown` | Contains `# `, `## `, `**text**`, `- [ ]`, `[text](url)` |
| `HtmlXml` | Starts with `<` and contains `</` closing tag |
| `PlainText` | Default fallback |

Priority: Credential > specific types > PlainText. If multiple match, pick highest confidence.

---

## Credential Detection Patterns

The detector in `src-tauri/src/clipboard/credential_detector.rs` must catch:

| Type | Regex Pattern |
|------|--------------|
| AWS Access Key | `AKIA[0-9A-Z]{16}` |
| AWS Secret Key | 40-char base64 near `aws_secret_access_key` |
| GitHub Token | `gh[ps]_[A-Za-z0-9_]{36,}` |
| GitLab Token | `glpat-[A-Za-z0-9\-]{20,}` |
| Anthropic API Key | `sk-ant-[A-Za-z0-9\-_]{40,}` |
| OpenAI API Key | `sk-[A-Za-z0-9]{48,}` |
| JWT Token | `eyJ[A-Za-z0-9\-_]+\.eyJ[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+` |
| Private Key | `-----BEGIN (RSA\|EC\|)PRIVATE KEY-----` |
| Connection w/ Password | `://[^:]+:[^@]+@` (password in URL) |
| Generic API Key | 32+ alphanum chars near keywords: api_key, apikey, token, secret, password, passwd, authorization |

**Masking**: Show first 4 + last 4 chars, replace middle with `••••••••`
**Auto-expiry**: Default 30 minutes, configurable

---

## Database Schema

```sql
-- Core table
CREATE TABLE clip_items (
    id TEXT PRIMARY KEY,                          -- UUID v4
    content TEXT NOT NULL,
    content_type TEXT NOT NULL DEFAULT 'PlainText',
    content_hash TEXT NOT NULL,                   -- SHA256 for dedup
    content_length INTEGER NOT NULL,
    is_credential BOOLEAN NOT NULL DEFAULT 0,
    credential_type TEXT,
    source_app TEXT,
    source_window_title TEXT,
    is_pinned BOOLEAN NOT NULL DEFAULT 0,
    is_starred BOOLEAN NOT NULL DEFAULT 0,
    expires_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_pasted_at TEXT,
    paste_count INTEGER NOT NULL DEFAULT 0,
    tags TEXT                                     -- JSON array
);

-- Indexes
CREATE INDEX idx_clip_created ON clip_items(created_at DESC);
CREATE INDEX idx_clip_type ON clip_items(content_type);
CREATE INDEX idx_clip_hash ON clip_items(content_hash);
CREATE INDEX idx_clip_pinned ON clip_items(is_pinned) WHERE is_pinned = 1;
CREATE INDEX idx_clip_source ON clip_items(source_app);

-- Full-text search
CREATE VIRTUAL TABLE clip_items_fts USING fts5(
    content, source_app, source_window_title, tags,
    content='clip_items', content_rowid='rowid'
);

-- FTS sync triggers (INSERT, UPDATE, DELETE)

-- Paste tracking
CREATE TABLE paste_events (
    id TEXT PRIMARY KEY,
    item_id TEXT NOT NULL REFERENCES clip_items(id) ON DELETE CASCADE,
    target_app TEXT,
    target_window_title TEXT,
    pasted_at TEXT NOT NULL DEFAULT (datetime('now')),
    session_id TEXT
);

-- Prediction frequency table
CREATE TABLE prediction_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type TEXT NOT NULL,
    source_app TEXT,
    target_app TEXT NOT NULL,
    frequency INTEGER NOT NULL DEFAULT 1,
    last_used_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Workflow chains
CREATE TABLE workflow_chains (
    id TEXT PRIMARY KEY,
    chain_hash TEXT NOT NULL UNIQUE,
    items_json TEXT NOT NULL,
    frequency INTEGER NOT NULL DEFAULT 1,
    last_triggered_at TEXT NOT NULL DEFAULT (datetime('now')),
    source_context TEXT
);

-- Settings
CREATE TABLE app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

---

## IPC Commands (Tauri invoke)

### Clipboard Commands
```
get_recent_items(limit: u32, offset: u32) -> Vec<ClipItem>
get_item(id: String) -> ClipItem
search_items(query: String, limit: u32) -> Vec<ClipItem>
delete_item(id: String) -> ()
toggle_pin(id: String) -> ()
toggle_star(id: String) -> ()
paste_item(id: String) -> ()
clear_history() -> ()
clear_expired_credentials() -> ()
```

### Prediction Commands
```
get_predictions(limit: u32) -> Vec<RankedItem>
```

### Settings Commands
```
get_all_settings() -> HashMap<String, String>
update_setting(key: String, value: String) -> ()
get_ignored_apps() -> Vec<String>
add_ignored_app(app_name: String) -> ()
remove_ignored_app(app_name: String) -> ()
```

### Events (Rust → Frontend)
```
clipboard:new-item      → ClipItem payload
clipboard:error         → String payload
workflow:chain-detected  → WorkflowChain payload
security:credential-detected → { itemId, credType } payload
settings:changed        → { key, value } payload
```

---

## TypeScript Types (src/lib/types.ts)

```typescript
export type ContentType =
  | 'Url' | 'Email' | 'IpAddress' | 'Json' | 'Yaml' | 'Sql'
  | 'ShellCommand' | 'Code' | 'AwsArn' | 'ConnectionString'
  | 'FilePath' | 'Credential' | 'Markdown' | 'HtmlXml' | 'PlainText';

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
  reason: string;
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

export type AIProvider = 'local' | 'openai' | 'anthropic' | 'ollama';

export interface AppSettings {
  maxHistoryItems: number;
  credentialAutoExpireMinutes: number;
  hotkeyQuickPaste: string;
  hotkeyHistory: string;
  theme: 'system' | 'light' | 'dark';
  showSourceContext: boolean;
  showTypeBadges: boolean;
  enablePredictions: boolean;
  enableWorkflowChains: boolean;
  enableSemanticSearch: boolean;
  aiProvider: AIProvider;
  ignoredApps: string[];
  startupOnLogin: boolean;
  overlayPosition: 'cursor' | 'center' | 'top-right';
  overlayMaxItems: number;
  dedupEnabled: boolean;
  dedupWindowSeconds: number;
}
```

---

## UI Specifications

### Quick Paste Overlay
- **Trigger**: Ctrl+Shift+V (Cmd+Shift+V on Mac)
- **Size**: 420px wide, max 480px tall
- **Position**: At cursor (configurable)
- **Items shown**: 8 (configurable)
- **Each item shows**: Type badge (colored pill) | Preview (first line, max 80 chars) | Source app | Relative time
- **Keyboard**: ↑↓ navigate, Enter paste, Esc close, Tab ghost-paste, typing filters
- **Pinned items**: Always at top, star icon
- **Credentials**: Masked preview, lock icon, red badge
- **Chain indicator**: "Chain: 3 items queued" badge when detected
- **Theme**: Dark/light following system, semi-transparent backdrop blur

### Prediction Scoring Formula
```
score = pin_boost * 100
      + chain_boost * 50
      + frequency_score * 0.4
      + recency_score * 0.3      // e^(-t/3600) decay, τ=1hour
      + type_match_score * 0.2
      + source_affinity * 0.1
```

---

## Verification Commands

```bash
# Rust
cargo check                     # Compile check
cargo clippy -- -D warnings     # Lint (treat warnings as errors)
cargo test                      # Unit tests
cargo fmt -- --check            # Format check

# Frontend
pnpm tsc --noEmit               # Type check
pnpm lint                       # ESLint
pnpm vitest run                 # Tests

# Full app
cargo tauri dev                 # Dev mode with hot reload
cargo tauri build               # Production build
```

---

## Build Phases

### Phase 1 (MVP): "Better Ditto" — Weeks 1-3
Clipboard monitoring, content classification, credential detection, SQLite storage with FTS5, Quick Paste overlay, History browser, Settings panel, system tray, pin/star, dark/light theme, dedup, source context.

### Phase 2 (Intelligence): "The Differentiator" — Weeks 4-6
Paste prediction engine, paste event tracking, workflow chain detection, chain queue in UI, ghost paste, credential auto-expiry.

### Phase 3 (AI): "The Moat" — Weeks 7-10
Local ONNX embeddings, sqlite-vec vector search, semantic search, BYOK API config, natural language search in overlay.

### Phase 4 (Launch): Weeks 11-12
CI/CD, auto-updater, README with demo GIF, performance optimization, security audit, launch on GitHub/HN/ProductHunt.

---

## Agent Team Instructions

When building with Claude Code agent teams, spawn 4 teammates:

1. **rust-backend**: All code in `src-tauri/src/`. Build order: storage → clipboard → prediction → commands → tray → ai
2. **react-frontend**: All code in `src/`. Build order: types → stores → hooks → shared components → QuickPaste → History → Settings → App.tsx
3. **integration-config**: All config files. Build order: Cargo.toml → package.json → tauri.conf.json → capabilities → vite/tailwind/ts configs → scripts → CI/CD → README
4. **quality-testing**: All tests. Wait for code, then write tests + review all code for rule violations.

**Teammates do NOT inherit conversation history. Include full context in spawn prompts.**
