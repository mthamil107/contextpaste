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
| **Distribution** | [GitHub Releases](https://github.com/mthamil107/contextpaste/releases) — exe, msi, dmg, deb, AppImage |
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

### Settings

Access via system tray > Settings, or click "Settings" in the nav bar:

| Tab | What you can configure |
|-----|----------------------|
| **General** | Max history (default 5000), theme (system/light/dark), overlay position (cursor/center/top-right), overlay max items, dedup toggle, type badges, source context |
| **Shortcuts** | View current hotkey bindings (Ctrl+Shift+V, Ctrl+Shift+H) |
| **Security** | Credential auto-expire duration (1-1440 min), clear expired credentials, clear all history |
| **AI** | Enable/disable predictions, AI provider selection (local/OpenAI/Anthropic/Ollama), semantic search toggle |

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
| Windows (installer) | [ContextPaste-setup.msi](https://github.com/mthamil107/contextpaste/releases/latest) | ~10MB |
| Windows (portable) | [ContextPaste-setup.exe](https://github.com/mthamil107/contextpaste/releases/latest) | ~10MB |
| macOS | [ContextPaste.dmg](https://github.com/mthamil107/contextpaste/releases/latest) | ~10MB |
| Linux (Debian/Ubuntu) | [contextpaste_amd64.deb](https://github.com/mthamil107/contextpaste/releases/latest) | ~10MB |
| Linux (AppImage) | [ContextPaste.AppImage](https://github.com/mthamil107/contextpaste/releases/latest) | ~10MB |

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

### Rust Tests (56 tests)

```bash
cd src-tauri
cargo test
```

Test coverage:
- **Classifier** — 14 tests covering all 15 content types
- **Credential detector** — 11 tests for each pattern + masking
- **Prediction engine** — 8 tests (pin, recency, frequency, type match, source affinity, full ranking, no-context fallback)
- **Workflow chains** — 7 tests (track, detect, hash, store, upsert, ordering, source matching)
- **Database** — 2 tests (init, migrations idempotency)
- **Queries** — 14 tests (CRUD, settings, dedup, paste history, prediction stats, source affinity)

### TypeScript Type Check

```bash
pnpm tsc --noEmit
```

### E2E Tests (Playwright)

```bash
pnpm test:e2e
```

Covers: app launch, navigation, theme toggle, Quick Paste overlay, History panel, Settings panel.

---

## Tech Stack

- **Framework**: [Tauri 2.x](https://v2.tauri.app) (Rust + WebView)
- **Frontend**: React 18 + TypeScript + Tailwind CSS + Zustand
- **Database**: SQLite (WAL mode) + FTS5 full-text search
- **UI Library**: cmdk pattern + Lucide icons
- **AI** (Phase 3): ONNX Runtime + sqlite-vec
- **Installer size**: ~10MB (vs 100MB+ for Electron apps)
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
├── SQLite Store (WAL, FTS5, migrations)
├── Global Shortcut Handler (Ctrl+Shift+V/H)
├── Credential Auto-Expiry Timer (60s interval)
└── System Tray

REACT FRONTEND (WebView via IPC)
├── Quick Paste Overlay (420px, keyboard nav)
├── History Browser (search + filters + detail)
├── Settings Panel (4 tabs)
├── Zustand Stores (clipboard, settings, UI)
└── Tauri IPC Wrappers (typed, centralized)
```

---

## Roadmap

- [x] **Phase 1** — Core MVP (clipboard, classification, credentials, storage, UI)
- [x] **Phase 2** — Intelligence (predictions, paste tracking, workflow chains, ghost paste)
- [ ] **Phase 3** — AI (ONNX embeddings, sqlite-vec, semantic search, BYOK)
- [ ] **Phase 4** — Launch (CI/CD, auto-updater, performance optimization)

---

## AI Configuration — BYOK (Bring Your Own Key)

ContextPaste works fully **without any AI setup or API key**. The built-in local ONNX model (all-MiniLM-L6-v2) ships with the app and handles semantic search for free.

For enhanced AI features, you can optionally use your own API key — we never charge for AI, you pay your provider directly:

**Settings > AI > Provider:**
- **Local** (default) — Free, built-in ONNX model, no internet needed, no key required
- **OpenAI** — `text-embedding-3-small` — bring your own OpenAI API key
- **Anthropic** — Claude for semantic parsing — bring your own Anthropic API key
- **Ollama** — Free, local LLM server (`nomic-embed-text`) — no key needed

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
