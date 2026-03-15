# ContextPaste

**AI-Powered Smart Clipboard Manager** — Free, Open Source, Cross-Platform

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Built with Tauri](https://img.shields.io/badge/Built%20with-Tauri%202-orange)](https://v2.tauri.app)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)]()
[![Price](https://img.shields.io/badge/Price-Free-brightgreen)]()
[![Sponsor](https://img.shields.io/badge/Sponsor-GitHub%20Sponsors-ea4aaa)](https://github.com/sponsors/user)

> The first clipboard manager that **learns where you paste what** — and predicts it next time.

<!-- TODO: Add demo GIF here -->
<!-- ![ContextPaste Demo](docs/demo.gif) -->

### Completely Free

| | |
|---|---|
| **License** | GPL v3 (same as Notepad++) |
| **Price** | Free — all features, all platforms, forever |
| **Local AI** | Built-in ONNX model ships with the app — no API key needed |
| **Cloud AI** | BYOK (Bring Your Own Key) — use your own OpenAI, Anthropic, or Ollama key |
| **Revenue** | [GitHub Sponsors](https://github.com/sponsors/user) + donations. Future: optional cloud sync service |
| **Distribution** | [GitHub Releases](https://github.com/mthamil107/contextpaste/releases) — exe, msi (Windows). macOS/Linux coming soon |
| **Telemetry** | None. Zero tracking. All data stays on your machine. |

---

## Why ContextPaste?

Every clipboard manager stores your history. **None of them learn from it.**

ContextPaste watches your copy-paste patterns and builds a prediction model:
- Copy an AWS ARN? It knows you'll paste it in your `.env` file.
- Start a deployment sequence? It queues the next 3 items you always copy.
- Need "that postgres connection string from Tuesday"? Just ask in natural language.

---

## Usage

### Getting Started

1. **Launch ContextPaste** — After installation, ContextPaste starts minimized in your system tray
2. **Copy anything** — It silently captures everything you copy, classifying content type automatically
3. **Press `Ctrl+Shift+V`** — The Quick Paste overlay appears with your recent clips, ranked by prediction
4. **Arrow keys + Enter** — Navigate and paste instantly. Or just start typing to search.

### Quick Paste Overlay

The overlay is your primary interface. Open it from any app with `Ctrl+Shift+V`:

```
┌─────────────────────────────────────┐
│ 🔍 Search clipboard...              │
├─────────────────────────────────────┤
│ 📌 URL  https://api.example.c...  2m│
│    JSON {"key": "value", "na...   5m│
│    SQL  SELECT * FROM users W... 12m│
│    Code const handler = asyn...  1h │
│    Shell docker compose up -d    2h │
├─────────────────────────────────────┤
│ ↑↓ navigate  ↵ paste  ⇥ ghost  esc │
└─────────────────────────────────────┘
```

- **Type to filter** — Search narrows results in real-time (FTS5 full-text search)
- **Enter** — Pastes the selected item and closes the overlay
- **Tab (Ghost Paste)** — Copies to clipboard without closing, so you can paste multiple items
- **Escape** — Closes the overlay without pasting

### Pin & Star Items

- **Pin** (from History detail view) — Pinned items always appear at the top of Quick Paste
- **Star** — Starred items get a score boost in predictions

### History Browser

Open with `Ctrl+Shift+H` or click "History" in the nav bar:

- **Search** — Full-text search across all clipboard content
- **Filter by type** — Click filter pills (All, URLs, Code, JSON, SQL, Shell, Secrets)
- **Detail view** — Click any item to see full content, paste count, source app, timestamps
- **Actions** — Pin, star, copy, or delete from the detail panel

### Credential Handling

When you copy something that looks like an API key, token, or password:

1. ContextPaste detects it automatically (10 pattern types)
2. Content is **masked** in the UI: `ghp_••••••••xyz9`
3. A red "Secret" badge appears on the item
4. The item **auto-expires** after 30 minutes (configurable in Settings > Security)
5. Credentials are **never** included in search indexes or analytics

### Workflow Chains

ContextPaste watches for repeating copy patterns:

1. You copy a URL, then some code, then a SQL query — three times in a row
2. ContextPaste detects this as a "workflow chain"
3. Next time you start the pattern, a **"Chain: 3 items queued"** badge appears
4. Chain frequency is tracked — the more you repeat a pattern, the stronger the detection

### Prediction Intelligence

The prediction engine learns from your habits:

- **What you paste where** — If you always paste JSON into Postman and SQL into DBeaver, those items get boosted when those apps are focused
- **Source affinity** — Items from Chrome get ranked higher when you're in VS Code, if that's your pattern
- **Recency decay** — Recent copies score higher, with exponential decay (half-life: ~1 hour)
- **Frequency** — Items you paste often get a persistent boost

### Semantic Search

Search your clipboard history by meaning instead of exact text. Enable in **Settings > AI**.

- **Local mode** (default) — Works offline with the bundled ONNX model. No setup required.
- **OpenAI mode** — Enter your API key in Settings > AI. Uses `text-embedding-3-small` for higher-dimensional embeddings.
- **Ollama mode** — Install [Ollama](https://ollama.ai), run `ollama pull nomic-embed-text`, then select Ollama in Settings > AI. Runs entirely on your machine.

**Re-indexing**: If you enable semantic search after already having clipboard history, click "Re-index All Items" in Settings > AI to generate embeddings for all existing items.

**Search naturally**: Instead of remembering exact text, try queries like:
- "that postgres connection string" instead of `postgres://`
- "the AWS key I copied this morning" instead of `AKIA...`
- "docker command to restart services" instead of `docker compose restart`

### Context-Aware Smart Paste (The Killer Feature)

When you press `Ctrl+Shift+V`, ContextPaste uses **3 intelligence layers** to bring the right item to the top:

#### Layer 1: OCR Screen Reading
Takes a screenshot near your cursor, runs Windows built-in OCR, and reads what the app is asking for:

```
Terminal shows: "GitHub Username:"     → OCR reads "Username"  → PlainText items ranked first
Terminal shows: "Access Token:"        → OCR reads "Token"     → Credential items ranked first
Terminal shows: "Docker Hub password:" → OCR reads "password"  → Credential items ranked first
```

**Real OCR captures from testing** (actual log output):
```
OCR: "Enter Docker Hub oas sword/ token :"  → matched Credential → score 377.9
OCR: "GitHub Username: mthami1107"          → matched PlainText  → score 390.6
OCR: "remote: Invalid username or token"    → matched PlainText  → score 390.6
```

The OCR runs in a **background thread** (~1 second) — the overlay shows instantly, then re-ranks items when OCR completes.

#### Layer 2: Paste Sequence Tracking
Learns the ORDER you paste things. If you always paste `mthamil107` then `ghp_token`, after pasting username the token automatically moves to position #1:

```
Paste #1: mthamil107 (Username)
  → System records: "after mthamil107, user pasted ghp_token"

Next Ctrl+Shift+V:
  → System checks last paste was mthamil107
  → Boosts ghp_token to top (+200 score)
  → Token is at position #1, just press Enter
```

This works for ANY repeating workflow — deploy scripts, form filling, configuration sequences.

#### Layer 3: Frequency-Based Ranking
Items you paste frequently rank higher than items you just copied once:

```
mthamil107  (11 pastes) → score: 66.0 (frequency)
ghp_token   (3 pastes)  → score: 18.0 (frequency)
random text (0 pastes)  → score: 0.0  (frequency)
```

Frequency weight is 60% of the base score — much stronger than recency (20%).

#### Combined Scoring (all layers together)

| Signal | Weight | Description |
|--------|--------|-------------|
| **Sequence boost** | +200 | "You always paste B after A" — strongest signal |
| **OCR screen match** | +150 | Screen says "token" → boosts Credential items |
| **Pinned items** | +100 | User-pinned items always near top |
| **Content word match** | +80 | Screen text words found in item content |
| **Paste frequency** | ×0.6 | Items pasted many times score higher |
| **Recency** | ×0.2 | Newer items get a smaller boost |
| **Starred items** | +10 | User-starred items get a small boost |

#### How to Use

1. **First time**: Just use `Ctrl+Shift+V` → arrow to select → `Enter` to paste
2. **After a few uses**: Items you paste often will automatically rank higher
3. **After pasting in sequence**: The next item in your workflow moves to #1
4. **OCR context**: If the screen says "password" or "token", credentials rank first automatically

No configuration needed — it learns from your behavior.

#### Enable Auto-Paste (Optional)

For fully automatic pasting when confidence is high:
1. Settings > Auto-Paste > Enable auto-paste ON
2. Set confidence threshold (default 75%)
3. When confident → pastes directly + shows toast notification
4. When not confident → shows overlay for manual selection

**Safety:**
- Low confidence always falls back to the overlay — no guessing
- Feature is **opt-in** (disabled by default)

### Paste Rules

Create custom rules in **Settings > Auto-Paste > Paste Rules** for predictable scenarios:

**Example rules:**

| Rule Name | App Pattern | Context Pattern | Action |
|-----------|------------|-----------------|--------|
| Git Token | `Terminal\|cmd\|powershell` | `password\|token\|credential` | Paste most recent Credential |
| Docker Login | `Docker\|Terminal` | `docker.*token\|docker.*login` | Paste most recent Credential |
| DB Connection | `DBeaver\|pgAdmin` | `connection\|connect\|jdbc` | Paste most recent ConnectionString |
| API Endpoint | `Postman\|Insomnia` | `url\|endpoint\|base.*url` | Paste most recent Url |

Rules use **regex patterns** and support:
- **App pattern** — match against the application name
- **Window title pattern** — match against the window title
- **Context pattern** — match against the text the app is asking for
- **Content type filter** — match against specific content types
- **Priority ordering** — higher priority rules are checked first
- **Enable/disable toggle** — temporarily disable rules without deleting
- **Trigger tracking** — see how many times each rule has been used

### Settings

Access via system tray > Settings, or click "Settings" in the nav bar:

| Tab | What you can configure |
|-----|----------------------|
| **General** | Max history (default 5000), theme (system/light/dark), overlay position (cursor/center/top-right), overlay max items, dedup toggle, type badges, source context |
| **Shortcuts** | View current hotkey bindings (Ctrl+Shift+V, Ctrl+Shift+H) |
| **Security** | Credential auto-expire duration (1-1440 min), clear expired credentials, clear all history |
| **AI** | Enable/disable predictions, AI provider selection (local/OpenAI/Ollama), semantic search toggle |
| **Auto-Paste** | Enable/disable auto-paste, confidence threshold slider (50-95%), toast notifications, paste rules manager |

### System Tray

Right-click the ContextPaste tray icon for quick actions:

- **Quick Paste** — Opens the overlay
- **History** — Opens history browser
- **Settings** — Opens settings panel
- **Quit** — Exits ContextPaste

---

## Implemented Features

### Phase 1 — Core MVP

#### Clipboard Monitoring & Storage
- **Real-time clipboard capture** — Silently monitors clipboard changes in a background thread with 500ms polling
- **SHA-256 deduplication** — Identical content is never stored twice; configurable dedup window
- **SQLite with WAL mode** — High-performance storage with write-ahead logging for concurrent reads
- **FTS5 full-text search** — Instant search across all clipboard content, source apps, and tags
- **History limit enforcement** — Automatic cleanup of oldest non-pinned items (default: 5000 max)

#### Content Classification (15 Types)
Every clipboard item is automatically classified using regex-based detection:

| Type | Detection | Example |
|------|-----------|---------|
| URL | `http://`, `https://`, `www.` prefix | `https://api.example.com/v2` |
| Email | Standard email regex | `user@example.com` |
| IP Address | IPv4 `\d.\d.\d.\d` or IPv6 | `192.168.1.1`, `::1` |
| JSON | Starts with `{`/`[` + valid parse | `{"key": "value"}` |
| YAML | `---` or multi-line `key: value` | `apiVersion: v1` |
| SQL | `SELECT`, `INSERT`, `UPDATE`, etc. | `SELECT * FROM users` |
| Shell Command | `git`, `docker`, `npm`, `curl`, etc. | `kubectl get pods` |
| Code | `function`, `class`, `import`, `=>`, etc. | `const x = () => {}` |
| AWS ARN | `arn:aws:` prefix | `arn:aws:s3:::my-bucket` |
| Connection String | Protocol `://` (postgres, redis, etc.) | `redis://localhost:6379` |
| File Path | Unix `/path` or Windows `C:\path` | `/usr/local/bin/node` |
| Credential | API keys, tokens, passwords (see below) | `ghp_abc...` |
| Markdown | `# `, `**bold**`, `[link](url)` | `# Heading` |
| HTML/XML | `<tag>...</tag>` | `<div>Hello</div>` |
| Plain Text | Default fallback | `hello world` |

Priority: Credential > Specific types > Plain Text.

#### Credential Detection & Security
Auto-detects 10 credential patterns and treats them specially:

| Credential Type | Pattern |
|----------------|---------|
| AWS Access Key | `AKIA[0-9A-Z]{16}` |
| AWS Secret Key | 40-char base64 near `aws_secret_access_key` |
| GitHub Token | `gh[ps]_[A-Za-z0-9_]{36,}` |
| GitLab Token | `glpat-[A-Za-z0-9-]{20,}` |
| Anthropic API Key | `sk-ant-[A-Za-z0-9-_]{40,}` |
| OpenAI API Key | `sk-[A-Za-z0-9]{48,}` |
| JWT Token | `eyJ...` three-segment base64 |
| Private Key | `-----BEGIN (RSA\|EC) PRIVATE KEY-----` |
| Connection w/ Password | `://user:pass@host` |
| Generic API Key | 32+ chars near `api_key`, `token`, `secret`, etc. |

Security measures:
- **Masked display** — First 4 + last 4 chars visible, middle replaced with `••••••••`
- **Auto-expiry** — Credentials automatically deleted after 30 minutes (configurable)
- **Background cleanup** — Expiry timer runs every 60 seconds
- **Never in embeddings** — Credentials excluded from AI/vector search index

#### Quick Paste Overlay
- **Hotkey**: `Ctrl+Shift+V` (global, works from any app)
- **420px wide, max 480px tall** — Compact overlay with backdrop blur
- **Keyboard-first UX**: `↑↓` navigate, `Enter` paste, `Esc` close, type to search
- **Prediction-ranked** — Items sorted by prediction score, not just recency
- **Pinned items at top** — Pin important clips for quick access
- **Content type badges** — Color-coded pills showing item type
- **Relative timestamps** — "2 minutes ago", "1 hour ago"
- **Source app context** — Shows which app the content was copied from

#### History Browser
- **Hotkey**: `Ctrl+Shift+H`
- **Full-text search** with debounced input (150ms)
- **Type filters** — Filter by URL, Code, JSON, SQL, Shell, Secrets, or All
- **Detail panel** — Click any item to see full content, metadata, paste count
- **Inline actions** — Pin, star, copy, delete from the detail view

#### Settings Panel (4 tabs)
- **General** — Max history, theme (system/light/dark), overlay position, dedup toggle
- **Shortcuts** — View current hotkey bindings
- **Security** — Credential auto-expire duration, clear expired, clear all history
- **AI** — Prediction toggle, AI provider selection, semantic search toggle

#### System Tray
- **Quick Paste** — Opens overlay
- **History** — Opens history browser
- **Settings** — Opens settings panel
- **Quit** — Exits the application

#### Dark/Light Theme
- **CSS variable-based** theming with Tailwind `dark:` classes
- **System preference detection** — Auto-follows OS theme when set to "system"
- **Toggle button** — Sun/moon icon in the nav bar

---

### Phase 2 — Intelligence

#### Paste Event Tracking
- **Records every paste** — Logs which item was pasted, into which app, at what time
- **Active window detection** — On Windows, uses PowerShell to detect the foreground app name and window title via Win32 `GetForegroundWindow`
- **Prediction stats** — Tracks `content_type → target_app` frequency for prediction scoring
- **Paste history per item** — Query paste history for any clip item (`get_paste_history` command)

#### Enhanced Prediction Engine
Scoring formula with 6 factors:

```
score = pin_boost * 100           // Pinned items always on top
      + chain_boost * 50          // Active workflow chain items
      + frequency_score * 0.4     // How often you paste this item (0-100)
      + recency_score * 0.3       // Exponential decay: e^(-t/3600), τ=1 hour
      + type_match_score * 0.2    // Does this content type match what you usually paste here?
      + source_affinity * 0.1     // Do items from this source get pasted into this target app?
```

- **Type match scoring** — Queries `prediction_stats` to find which content types are most pasted into the current target app, boosts matching items
- **Source affinity** — If you often paste items from Chrome into VS Code, those items get boosted when VS Code is focused
- **Starred item boost** — Starred items get +10 score

#### Workflow Chain Detection
- **Sliding window** — Tracks last 10 copy events in a thread-safe `VecDeque`
- **Pattern detection** — Looks for repeating sequences of 3-5 content types (e.g., URL → Code → SQL repeated twice)
- **Source context matching** — Chains must come from the same source apps to be detected
- **Hash-based storage** — Chain patterns are SHA-256 hashed and upserted with frequency tracking
- **Emits events** — Frontend notified via `workflow:chain-detected` event when a chain is found

#### Ghost Paste (Tab Key)
- **Paste without closing** — Press `Tab` to copy selected item to clipboard without dismissing the overlay
- **Visual feedback** — Brief "Copied!" flash animation on the item
- **Multiple pastes** — Paste the same item repeatedly, or navigate and ghost-paste different items

#### Chain Queue UI
- **Chain indicator** — Shows "Chain: N items" badge when a workflow chain is active
- **Mini preview** — Displays content types in the chain as small badges
- **Auto-detection** — Listens for `workflow:chain-detected` events from the Rust backend

#### Global Shortcuts
- **Ctrl+Shift+V** — Opens Quick Paste overlay + shows/focuses window
- **Ctrl+Shift+H** — Opens History browser + shows/focuses window
- **Registered via `tauri-plugin-global-shortcut`** — Works from any application

#### Credential Auto-Expiry
- **Background timer** — Runs every 60 seconds in a dedicated thread
- **Automatic deletion** — Removes credential items past their `expires_at` timestamp
- **Configurable duration** — Default 30 minutes, adjustable in Settings > Security

---

### Phase 3 — AI

#### Local AI Embeddings (ONNX Runtime)
- **Bundled model** — Ships with `all-MiniLM-L6-v2` ONNX model, no download or setup required
- **384-dimensional embeddings** — Each clipboard item is converted to a semantic vector for meaning-based search
- **~15ms per item** — Fast enough to embed every copy without noticeable delay
- **Fully offline** — No internet connection needed in local mode

#### Semantic Search
- **Search by meaning, not just keywords** — Type "that email address from yesterday" instead of remembering exact text
- **Cosine similarity ranking** — Results scored by how semantically close they are to your query
- **Credential-safe** — Credentials are never embedded or included in the vector index

#### Hybrid Search
- **Automatic mode selection** — Short exact queries use FTS5 full-text search; natural language queries use semantic search
- **Best of both worlds** — Exact text matching when you know the content, AI-powered search when you don't
- **Seamless UX** — Works through the same search bar in Quick Paste overlay and History browser

#### BYOK (Bring Your Own Key) Embedding Providers
- **Local (default)** — Built-in ONNX model, 384D embeddings, no API key needed
- **OpenAI** — Uses `text-embedding-3-small`, 1536D embeddings, requires API key
- **Ollama** — Uses `nomic-embed-text`, 768D embeddings, runs locally via Ollama server

#### Background Embedding Pipeline
- **Async on copy** — Embeddings are generated asynchronously after each clipboard capture, never blocking the copy hot path
- **Non-blocking architecture** — The clipboard monitor captures content instantly; embedding happens in a background task
- **Re-index support** — "Re-index All Items" button in Settings to backfill embeddings for existing history

---

### Phase 4 — Context-Aware Smart Paste

#### OCR Screen Reading
- **Windows built-in OCR** — Captures a 600×100px screenshot near the cursor, runs `Windows.Media.Ocr` via PowerShell to extract text
- **Works for terminals** — Unlike UI Automation, OCR reads actual pixel text from any app including PowerShell, cmd, Windows Terminal, SSH sessions
- **Non-blocking** — OCR runs in a background thread (~1 second). Overlay shows instantly, then re-ranks when OCR completes
- **Fallback chain** — OCR → UI Automation → window title
- **Cross-platform stub** — Non-Windows platforms fall back to window title only

#### Paste Sequence Tracking
- **Learns paste order** — Records which item was pasted immediately after the current item across all paste events
- **Automatic boost** — After pasting item A, item B (most frequently pasted next) gets +200 score boost
- **SQL-based detection** — Uses rowid-based join to find the immediately next paste event after each occurrence
- **Works for any workflow** — Deploy scripts, form filling, configuration sequences

#### Frequency-Weighted Prediction Engine
- **Paste count × 10** — Each paste multiplies the frequency score by 10 (capped at 100)
- **60% weight** — Frequency is the strongest base signal, dominating over recency (20%)
- **Items pasted 5+ times** always rank above items just copied once

#### Context-Aware Re-Ranking
- **Keyword to content type** — Screen text "token/password/secret" → Credential, "username/login" → PlainText, "url/endpoint" → Url, etc.
- **10 keyword groups** mapped to content types (PlainText, Credential, Url, Email, IpAddress, Json, Sql, ShellCommand, FilePath, ConnectionString)
- **Direct content matching** — Words from the OCR text found in clipboard item content boost that item
- **Combined scoring** — Sequence (+200), OCR match (+150), pin (+100), content match (+80), frequency (×0.6), recency (×0.2)

#### Paste Rules Engine
- **Regex-based matching** — User-defined rules with regex patterns for app name, window title, and screen context
- **AND logic** — All non-null conditions must match for a rule to trigger
- **Priority ordering** — Rules checked in priority order (highest first)
- **Action types** — "Paste most recent by type" or "Paste specific item"
- **CRUD management** — Create, edit, delete, enable/disable rules via Settings > Auto-Paste
- **Trigger tracking** — Each rule tracks how many times it has been used

#### Learned Paste Patterns
- **Records every manual paste** — Captures WHERE (app, window, OCR text) + WHAT (content type, item)
- **Frequency tracking** — Patterns used multiple times rank higher for auto-paste
- **Promote to rules** — "Make Rule" button auto-creates a paste rule from a learned pattern
- **Settings UI** — View, promote, or delete learned patterns in Settings > Auto-Paste

#### Database Support
- **paste_rules table** — Stores rule definitions with patterns, actions, and metadata
- **auto_paste_events table** — Tracks auto-paste decisions for confidence calibration
- **learned_patterns table** — Records WHERE + WHAT for every manual paste
- **Migrations v3 + v4** — Automatic schema upgrade on first launch

#### Credentials Persist
- **No auto-expiry** — Credentials stay in history like normal items for easy re-use
- **Masked display** — Content shown as `ghp_••••••••wxyz` in the UI
- **Red "Secret" badge** — Visual indicator for credential items
- **Never auto-pasted** — Always shows overlay for credentials (safety)

---

## Keyboard Shortcuts

| Action | Windows/Linux | macOS |
|--------|--------------|-------|
| Quick Paste | `Ctrl+Shift+V` | `Cmd+Shift+V` |
| History Browser | `Ctrl+Shift+H` | `Cmd+Shift+H` |
| Navigate items | `↑` `↓` | `↑` `↓` |
| Paste selected | `Enter` | `Enter` |
| Ghost paste | `Tab` | `Tab` |
| Close overlay | `Escape` | `Escape` |
| Search | Just start typing | Just start typing |

---

## Installation

### Download

| Platform | Download | Size |
|----------|----------|------|
| Windows (installer, NSIS) | [ContextPaste_0.3.0_x64-setup.exe](https://github.com/mthamil107/contextpaste/releases/latest) | ~82MB |
| Windows (installer, MSI) | [ContextPaste_0.3.0_x64_en-US.msi](https://github.com/mthamil107/contextpaste/releases/latest) | ~85MB |
| Windows (portable) | [ContextPaste-0.3.0-portable.exe](https://github.com/mthamil107/contextpaste/releases/latest) | ~15MB |
| macOS | Coming soon | — |
| Linux (Debian/Ubuntu) | Coming soon | — |
| Linux (AppImage) | Coming soon | — |

> **Note**: Installer size includes the bundled ONNX AI model (~87MB). The portable exe does not include the model — use an installer for full AI/semantic search functionality.

### Build from Source

```bash
# Prerequisites: Rust 1.77+, Node.js 18+, pnpm
git clone https://github.com/mthamil107/contextpaste.git
cd contextpaste
pnpm install
cargo tauri dev      # Development with hot reload
cargo tauri build    # Production build
```

---

## Testing

ContextPaste has **112 tests** across three layers: 56 Rust unit tests, 56 Playwright E2E tests, and TypeScript type checking.

### Prerequisites

```bash
# Install dependencies
pnpm install

# Install Playwright browsers (first time only)
npx playwright install chromium

# Download ONNX model (for AI tests)
bash scripts/download-model.sh
```

### Quick Test Commands

```bash
# Run everything
cd src-tauri && cargo test && cd .. && pnpm tsc --noEmit && pnpm test:e2e

# Individual test layers
cargo test                    # Rust unit tests (56 tests, ~0.1s)
pnpm tsc --noEmit             # TypeScript type check
pnpm test:e2e                 # Playwright E2E tests (56 tests, ~30s)
```

---

### Rust Unit Tests (56 tests)

```bash
cd src-tauri
cargo test
```

All Rust tests use in-memory SQLite databases (`init_test_db()`) — no files or cleanup needed.

#### Content Classifier (`clipboard/classifier.rs`) — 14 tests

| Test | What It Verifies |
|------|-----------------|
| `test_url` | Detects `http://`, `https://`, `www.` URLs |
| `test_email` | Detects standard email addresses |
| `test_ip` | Detects IPv4 addresses like `192.168.1.1` |
| `test_json` | Detects valid JSON starting with `{` or `[` |
| `test_yaml` | Detects YAML with `---` or `key: value` patterns |
| `test_sql` | Detects SELECT, INSERT, UPDATE, DELETE, CREATE, ALTER, DROP |
| `test_shell` | Detects commands starting with `git`, `docker`, `npm`, `curl`, etc. |
| `test_code` | Detects `function`, `class`, `import`, `const`, `=>`, `pub fn` |
| `test_aws_arn` | Detects `arn:aws:` prefixed ARNs |
| `test_connection_string` | Detects `postgres://`, `redis://`, `mongodb://` URIs |
| `test_filepath` | Detects Unix `/path/to/file` and Windows `C:\path` |
| `test_markdown` | Detects `# heading`, `**bold**`, `[link](url)` |
| `test_html` | Detects `<tag>...</tag>` patterns |
| `test_plain_text` | Falls back to PlainText for unrecognized content |

```bash
cargo test classifier::tests     # Run classifier tests only
```

#### Credential Detector (`clipboard/credential_detector.rs`) — 11 tests

| Test | What It Verifies |
|------|-----------------|
| `test_aws_access_key` | Detects `AKIA[0-9A-Z]{16}` |
| `test_github_token` | Detects `ghp_` and `ghs_` tokens |
| `test_gitlab_token` | Detects `glpat-` tokens |
| `test_anthropic_key` | Detects `sk-ant-` API keys |
| `test_openai_key` | Detects `sk-` OpenAI keys (48+ chars) |
| `test_jwt` | Detects `eyJ...` three-segment JWT tokens |
| `test_private_key` | Detects `-----BEGIN PRIVATE KEY-----` |
| `test_connection_with_password` | Detects `://user:pass@host` patterns |
| `test_generic_api_key` | Detects 32+ char strings near `api_key`, `token`, etc. |
| `test_no_credential` | Verifies normal text doesn't false-positive |
| `test_mask` / `test_mask_content` | Verifies masking: `ghp_abcd••••••••wxyz` |

```bash
cargo test credential_detector::tests
```

#### Prediction Engine (`prediction/engine.rs`) — 8 tests

| Test | What It Verifies |
|------|-----------------|
| `test_pinned_higher_score` | Pinned items score 100+ higher than unpinned |
| `test_recent_higher_than_old` | Items from 1 minute ago outscore items from 2 hours ago |
| `test_frequency_boost` | Items pasted 50 times score higher than items pasted once |
| `test_type_match_boosts_matching_content_type` | JSON items score higher when JSON is the most-pasted type into the target app |
| `test_source_affinity_boosts_correlated_source` | Items from Chrome score higher when Chrome→target is a frequent pattern |
| `test_full_ranking_order_with_mixed_items` | Verifies complete ranking: pinned > starred > recent > old |
| `test_no_target_app_still_works` | Prediction works even without active window context |
| `test_get_predictions_without_target_app` | Full prediction flow without target app returns valid results |

```bash
cargo test prediction::engine::tests
```

#### Workflow Chain Detection (`prediction/workflow.rs`) — 7 tests

| Test | What It Verifies |
|------|-----------------|
| `test_track_copy_event_adds_to_window` | Events are tracked in the sliding window |
| `test_track_copy_event_trims_to_max` | Window trims to max 10 events |
| `test_detect_chain_finds_repeating_pattern` | Detects URL→Code→SQL repeated twice |
| `test_detect_chain_no_match_without_repetition` | No false chain detection with non-repeating sequences |
| `test_detect_chain_requires_source_match` | Chains must come from the same source apps |
| `test_compute_chain_hash_deterministic` | Same pattern always produces the same SHA-256 hash |
| `test_store_chain_and_upsert` | Chains are stored and frequency is incremented on repeat |
| `test_get_top_chains_ordered_by_frequency` | Most frequent chains ranked first |

```bash
cargo test prediction::workflow::tests
```

#### Database & Queries — 16 tests

| Test | What It Verifies |
|------|-----------------|
| `test_init_db` | Database initializes with WAL mode and all tables |
| `test_migrations_idempotent` | Running migrations twice doesn't error |
| `test_insert_and_get` | Insert a ClipItem and retrieve it by ID |
| `test_delete_item` | Delete removes item and cascades |
| `test_toggle_pin` | Pin/unpin toggle works |
| `test_dedup_hash` | SHA-256 hash is consistent for same content |
| `test_settings` | get/update settings round-trip |
| `test_record_paste_increments_count` | Paste count increments on each paste event |
| `test_get_paste_history` | Paste events are recorded and queryable |
| `test_update_prediction_stat_increments_frequency` | Prediction stats track frequency |
| `test_get_type_match_scores` | Type match scores reflect paste patterns |
| `test_get_type_match_scores_empty` | Empty stats return empty scores |
| `test_get_source_affinity` | Source affinity scoring works |
| `test_get_source_affinity_no_data` | No data returns zero affinity |

```bash
cargo test storage::queries::tests
cargo test storage::database::tests
```

#### Running a Single Test

```bash
cargo test test_name              # Run by test name
cargo test classifier             # Run all tests with "classifier" in name
cargo test -- --nocapture         # Show println! output during tests
```

---

### TypeScript Type Check

```bash
pnpm tsc --noEmit
```

Validates all `.ts` and `.tsx` files compile without errors. Enforced rules:
- No `any` types anywhere
- All IPC calls typed through `src/lib/tauri.ts`
- All component props explicitly typed
- Strict null checks enabled

---

### E2E Tests — Playwright (56 tests)

```bash
pnpm test:e2e                    # Run all E2E tests (headless)
pnpm test:e2e -- --headed        # Run with visible browser
pnpm test:e2e -- --debug         # Step-through debugger
```

E2E tests run against the Vite dev server (auto-started on `localhost:1420`). They test the React frontend in isolation using Chromium.

#### App Launch (`tests/e2e/app-launch.spec.ts`) — 9 tests

| Test | What It Verifies |
|------|-----------------|
| `app loads and shows the main container` | Root `app-container` renders |
| `nav bar renders with all 3 view buttons` | Clipboard, History, Settings nav items visible |
| `default view is Quick Paste` | Quick Paste overlay shown on launch |
| `clicking History nav switches view` | History panel renders on click |
| `clicking Settings nav switches view` | Settings panel renders on click |
| `clicking Clipboard nav returns to Quick Paste` | Navigation back to Quick Paste works |
| `theme toggle button exists` | Sun/moon toggle button is visible |
| `theme toggle switches theme` | Clicking toggle changes `dark` class on `<html>` |
| `app container has correct structure` | Nav bar + content area both present |

```bash
pnpm test:e2e -- tests/e2e/app-launch.spec.ts
```

#### Quick Paste Overlay (`tests/e2e/quick-paste.spec.ts`) — 11 tests

| Test | What It Verifies |
|------|-----------------|
| `overlay renders` | Quick Paste container visible |
| `search bar is present` | Search input exists with placeholder |
| `item list container exists` | `clip-item-list` container renders |
| `shows empty state or items` | Handles both empty and populated states |
| `search input is focusable` | Search bar can receive keyboard focus |
| `typing in search filters content` | Input updates search state |
| `keyboard shortcut hints shown` | Footer shows ↑↓, ↵, ⇥, esc hints |
| `footer is visible` | Keyboard hints footer renders |
| `overlay has backdrop styling` | Overlay has proper CSS classes |
| `search clears on escape` | Escape key clears search input |
| `ghost paste hint shown` | Tab/ghost paste instruction visible |

```bash
pnpm test:e2e -- tests/e2e/quick-paste.spec.ts
```

#### History Panel (`tests/e2e/history.spec.ts`) — 10 tests

| Test | What It Verifies |
|------|-----------------|
| `history panel renders` | Panel container visible after nav click |
| `search bar exists in history` | History search input present |
| `filter bar renders` | Type filter bar visible |
| `filter bar has All button` | "All" filter button exists |
| `filter bar has type-specific buttons` | URL, Code, JSON, SQL, Shell, Secrets filters |
| `clicking a filter button highlights it` | Active filter gets visual highlight |
| `search input accepts text` | Can type in history search |
| `empty state or items displayed` | Handles empty/populated states |
| `history items show in a list` | Item list container renders |
| `detail panel shows on item click` | Clicking an item shows detail view |

```bash
pnpm test:e2e -- tests/e2e/history.spec.ts
```

#### Settings Panel (`tests/e2e/settings.spec.ts`) — 26 tests

**General Tab (7 tests)**:

| Test | What It Verifies |
|------|-----------------|
| `settings panel renders` | Panel container visible |
| `all 4 tabs are rendered` | General, Shortcuts, Security, AI tabs |
| `General tab content renders by default` | General settings shown on load |
| `max history items input exists` | Number input with min=100, max=50000 |
| `theme selector exists` | Dropdown with System, Light, Dark options |
| `overlay position selector exists` | At cursor, Center, Top right options |
| `dedup enabled checkbox exists` | Deduplication toggle checkbox |

**Shortcuts Tab (2 tests)**:

| Test | What It Verifies |
|------|-----------------|
| `Shortcuts tab renders` | Shortcuts content visible on tab click |
| `shortcut bindings are displayed` | Shows Ctrl+Shift+V and Ctrl+Shift+H |

**Security Tab (4 tests)**:

| Test | What It Verifies |
|------|-----------------|
| `Security tab renders` | Security content visible |
| `credential auto-expire input exists` | Number input with min=1, max=1440 |
| `clear expired credentials button exists` | Button present and clickable |
| `clear all history button exists` | Red-styled danger button present |

**AI Tab (13 tests)**:

| Test | What It Verifies |
|------|-----------------|
| `AI tab renders` | AI settings content visible |
| `enable predictions checkbox` | Predictions toggle works |
| `AI provider selector has 3 options` | Local (ONNX), OpenAI, Ollama |
| `selecting OpenAI shows API key field` | API key input appears for OpenAI |
| `selecting Ollama shows base URL field` | Base URL input appears for Ollama |
| `selecting Local hides API key field` | API key input hidden for local provider |
| `switching back to Local hides fields` | Fields disappear when switching back |
| `non-local provider shows Test button` | Save & Test Connection button visible |
| `semantic search checkbox is clickable` | Checkbox is enabled and toggleable |
| `Re-index All Items button exists` | Backfill button visible |
| `show type badges checkbox` | Type badge toggle works |
| `show source context checkbox` | Source context toggle works |
| `overlay max items input` | Number input with min=3, max=20 |

```bash
pnpm test:e2e -- tests/e2e/settings.spec.ts
```

#### Running Specific Tests

```bash
# Run a single test by name
pnpm test:e2e -- -g "theme toggle switches theme"

# Run a single test file
pnpm test:e2e -- tests/e2e/quick-paste.spec.ts

# Run with specific browser
pnpm test:e2e -- --project=chromium

# Generate HTML report
pnpm test:e2e -- --reporter=html
npx playwright show-report
```

---

### Manual Testing Checklist

For features that require the full Tauri app (clipboard monitoring, global shortcuts, system tray), test manually:

#### Clipboard Monitoring
```
1. Run `cargo tauri dev`
2. Copy text from any app — verify it appears in Quick Paste overlay
3. Copy a URL — verify it's classified as "URL" with blue badge
4. Copy JSON — verify it's classified as "JSON" with green badge
5. Copy the same text twice — verify dedup (only 1 entry)
```

#### Credential Detection
```
1. Copy a GitHub token: ghp_abcdefghijklmnopqrstuvwxyz1234567890
2. Verify red "Secret" badge appears
3. Verify content is masked: ghp_••••••••7890
4. Wait 30 minutes (or set auto-expire to 1 minute) — verify auto-deletion
5. Copy a JWT: eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.abc123
6. Verify it's detected and masked
```

#### Prediction Engine
```
1. Copy several items, paste them into different apps
2. Open Quick Paste in the same target app — verify recently-pasted items rank higher
3. Pin an item — verify it always appears at top
4. Star an item — verify it gets a score boost
```

#### Workflow Chains
```
1. Copy URL → Code → SQL from the same app, 3 times in sequence
2. Check console/logs for "Chain detected" event
3. Verify "Chain: 3 items" badge appears in Quick Paste overlay
```

#### Semantic Search (Phase 3)
```
1. Settings > AI > ensure "Enable semantic search" is checked
2. For local mode: model loads automatically on first search
3. Copy several items with different topics (email, code, URL)
4. Open Quick Paste and type a natural language query:
   - "that email address" → should surface email items
   - "database connection" → should surface connection strings
5. Short exact queries ("SELECT") should use FTS, not semantic search
6. Verify "AI" sparkle badge appears next to search bar during semantic search
```

#### BYOK Provider Testing
```
# OpenAI
1. Settings > AI > Select "OpenAI"
2. Enter your API key (sk-...)
3. Click "Save & Test Connection" — should show success
4. Click "Re-index All Items" — should show "Indexed N items"

# Ollama
1. Install Ollama and run: ollama pull nomic-embed-text && ollama serve
2. Settings > AI > Select "Ollama"
3. Base URL defaults to http://localhost:11434
4. Click "Save & Test Connection" — should show success

# Local (default)
1. Settings > AI > Select "Local (ONNX)"
2. No API key needed — works immediately
3. Status should show "AI Ready" with all-MiniLM-L6-v2 model
```

#### Auto-Paste (Phase 4)
```
1. Settings > Auto-Paste > enable "Enable auto-paste"
2. Set confidence threshold to 50% (for easier testing)
3. Copy a URL like https://example.com
4. Open Notepad, type "Enter URL:" then place cursor after it
5. Press Ctrl+Shift+V — should auto-paste the URL with toast notification
6. Copy a credential (ghp_abc...) — verify it shows overlay instead of auto-pasting
7. Create a paste rule:
   - Name: "Test Rule"
   - App pattern: notepad
   - Context pattern: password|token
   - Action: Paste most recent Credential
8. In Notepad, type "Enter password:" and press Ctrl+Shift+V
9. Verify the rule triggers and auto-pastes
10. Check Settings > Auto-Paste — rule should show "Triggered 1 time"
```

#### Paste Rules Management
```
1. Settings > Auto-Paste > click "Add Rule"
2. Fill in: Name, App pattern (regex), Context pattern (regex)
3. Select action type and content type
4. Click "Save Rule" — rule appears in the list
5. Click toggle icon to disable/enable the rule
6. Click trash icon to delete the rule
7. Verify disabled rules don't trigger on Ctrl+Shift+V
```

#### Global Shortcuts
```
1. Run `cargo tauri dev`
2. Switch to any other app (e.g., Notepad)
3. Press Ctrl+Shift+V — ContextPaste window should appear with Quick Paste
4. Press Escape — overlay closes
5. Press Ctrl+Shift+H — ContextPaste window should appear with History
```

#### System Tray
```
1. Right-click ContextPaste tray icon
2. Click "Quick Paste" — opens overlay
3. Click "History" — opens history browser
4. Click "Settings" — opens settings panel
5. Click "Quit" — app exits
```

---

### CI/CD Testing

The GitHub Actions workflow (`.github/workflows/ci.yml`) runs on every push/PR:

```yaml
# What CI runs:
- cargo check          # Compile check
- cargo clippy          # Lint (warnings = errors)
- cargo test            # Unit tests
- cargo fmt --check     # Format check
- pnpm tsc --noEmit     # TypeScript type check
- pnpm lint             # ESLint
```

---

### Test Statistics

| Layer | Tests | Speed | What It Covers |
|-------|-------|-------|---------------|
| Rust unit tests | 56 | ~0.1s | Classifier, credentials, prediction, workflow, DB, queries |
| TypeScript check | — | ~3s | Type safety across all frontend code |
| Playwright E2E | 56 | ~30s | UI rendering, navigation, settings, interactions |
| **Total** | **112** | **~35s** | Full stack coverage |

---

## Tech Stack

- **Framework**: [Tauri 2.x](https://v2.tauri.app) (Rust + WebView)
- **Frontend**: React 18 + TypeScript + Tailwind CSS + Zustand
- **Database**: SQLite (WAL mode) + FTS5 full-text search
- **UI Library**: cmdk pattern + Lucide icons
- **AI**: ONNX Runtime (all-MiniLM-L6-v2) + cosine similarity search
- **Installer size**: ~82MB with bundled AI model (portable exe: ~15MB)
- **Memory usage**: ~50MB (vs 200MB+ for Electron apps)

---

## Architecture

```
TAURI SHELL (Rust)
├── Clipboard Monitor (arboard, 500ms polling)
├── Content Classifier (15 types, regex-based)
├── Credential Detector (10 patterns + masking)
├── Prediction Engine (6-factor scoring)
├── Workflow Tracker (sliding window chain detection)
├── Context-Aware Smart Paste
│   ├── OCR Screen Reader (Windows.Media.Ocr, 600×100px near cursor)
│   ├── Paste Sequence Tracker (learns paste order, +200 boost)
│   ├── Context Re-Ranker (keyword→type matching, 10 keyword groups)
│   ├── Paste Rules Engine (regex-based rule matching)
│   └── Learned Patterns (records WHERE+WHAT, auto-rule creation)
├── SQLite Store (WAL, FTS5, 3 schema migrations)
├── Global Shortcut Handler (Ctrl+Shift+V/H)
├── Credential Auto-Expiry Timer (60s interval)
├── AI Embedding Pipeline
│   ├── ONNX Runtime (all-MiniLM-L6-v2, local)
│   ├── OpenAI API Client (text-embedding-3-small, BYOK)
│   ├── Ollama Client (nomic-embed-text, local server)
│   ├── Async Embedding on Copy (non-blocking background task)
│   └── Cosine Similarity Search (in-Rust vector scoring)
└── System Tray

REACT FRONTEND (WebView via IPC)
├── Quick Paste Overlay (420px, keyboard nav)
├── Auto-Paste Toast (confidence notification)
├── History Browser (search + filters + detail)
├── Settings Panel (5 tabs: General, Shortcuts, Security, AI, Auto-Paste)
├── Paste Rules Manager (CRUD with regex patterns)
├── Zustand Stores (clipboard, settings, UI)
└── Tauri IPC Wrappers (typed, centralized)
```

---

## Roadmap

- [x] **Phase 1** — Core MVP (clipboard, classification, credentials, storage, UI)
- [x] **Phase 2** — Intelligence (predictions, paste tracking, workflow chains, ghost paste)
- [x] **Phase 3** — AI (ONNX embeddings, semantic search, BYOK, hybrid search)
- [x] **Phase 4** — Context-Aware Auto-Paste (screen reading, paste rules, confidence scoring)
- [ ] **Phase 5** — Launch (CI/CD, auto-updater, performance optimization, macOS/Linux builds)

---

## AI Configuration — BYOK (Bring Your Own Key)

ContextPaste works fully **without any AI setup or API key**. The built-in local ONNX model (all-MiniLM-L6-v2) ships with the app and handles semantic search for free.

For enhanced AI features, you can optionally use your own API key — we never charge for AI, you pay your provider directly.

### Provider Setup

#### Local (Default)
No setup needed. The ONNX model ships with the app.
- **Model**: all-MiniLM-L6-v2
- **Embedding dimensions**: 384
- **Speed**: ~15ms per item
- **Requirements**: None (works offline)

#### OpenAI
1. Get an API key from [platform.openai.com](https://platform.openai.com)
2. Open **Settings > AI** and select "OpenAI" as the provider
3. Enter your API key in the field that appears
- **Model**: text-embedding-3-small
- **Embedding dimensions**: 1536
- **Requirements**: Internet connection, valid API key
- **Cost**: Per OpenAI pricing (~$0.02 per 1M tokens)

#### Ollama
1. Install [Ollama](https://ollama.ai) for your platform
2. Pull the embedding model: `ollama pull nomic-embed-text`
3. Make sure Ollama is running (`ollama serve`)
4. Open **Settings > AI** and select "Ollama" as the provider
- **Model**: nomic-embed-text
- **Embedding dimensions**: 768
- **Requirements**: Ollama running locally (default: `http://localhost:11434`)
- **Cost**: Free (runs on your hardware)

Your API key is stored locally and encrypted. It is never sent anywhere except directly to the provider you configure. ContextPaste has zero telemetry and zero cloud services.

---

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

```bash
# Development setup
git clone https://github.com/mthamil107/contextpaste.git
cd contextpaste
pnpm install
cargo tauri dev

# Run tests
cd src-tauri && cargo test    # Rust tests
cd .. && pnpm tsc --noEmit    # TypeScript check
pnpm test:e2e                 # E2E tests
```

---

## License

ContextPaste is free software, licensed under the [GNU General Public License v3.0](LICENSE) — the same license used by Notepad++, WordPress, and Linux.

You are free to use, modify, and distribute this software. If you distribute modified versions, they must also be GPL v3.

---

## Support the Project

ContextPaste is and will always be **completely free**. If you find it useful, consider supporting development:

- [GitHub Sponsors](https://github.com/sponsors/user)
- Star this repo
- Share with colleagues
- Report bugs and suggest features

---

## Acknowledgments

Built by [Thamilvendhan](https://github.com/user)

Powered by: [Tauri](https://tauri.app) | [React](https://react.dev) | [SQLite](https://sqlite.org) | [ONNX Runtime](https://onnxruntime.ai)
