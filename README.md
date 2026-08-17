# KnowteQuiz

> A local-first Markdown knowledge quiz app with AI-powered blind-spot diagnosis.

**KnowteQuiz** turns your local Markdown notes into an interactive learning environment. It doesn't just ask questions — it diagnoses *why* you got them wrong through multi-turn Socratic dialogues, pinpointing exact knowledge gaps like "concept confusion" or "logical leaps."

---

## Features

- **Obsidian-grade reading** — MarkdownIt + KaTeX + syntax highlighting for immersive note reading.
- **Dual quiz modes**
  - **Basic Mode**: Quick self-test with instant scoring and mistake logging.
  - **Advanced Mode**: Submit your reasoning → AI asks follow-up questions → Receive a detailed blind-spot report.
- **Purely local** — All data (notes, mistakes, settings) stays on your machine. Connect to any OpenAI-compatible endpoint (Ollama by default).
- **Three-pane workspace** — Explorer | Reader | Quiz & Diagnosis panel, with resizable columns and toggle shortcuts (`Ctrl+B` / `Ctrl+Shift+B`).
- **Cross-platform** — Built with Tauri v2 for Windows, macOS, and Linux.
- **Dual runtime** — Same app runs as a Tauri desktop window **or** as a standalone web server (Axum).

---

## Quick Start

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (Cargo) — required for desktop mode only

### First-Time Setup

```bash
git clone https://github.com/yourname/knowtequiz.git
cd knowtequiz
node setup.cjs          # Interactive installer: check env → install deps → build → choose mode
```

### Run

```bash
node start.cjs              # Desktop app (auto-builds if needed)
node start.cjs --web        # Web server on http://localhost:14200
node start.cjs --dev        # Development mode with hot-reload
node start.cjs --both       # Desktop + Web simultaneously
```

### Build Release

```bash
node start.cjs --build      # Or: npm run tauri:build
```

---

## Development

| Task | Command |
|------|---------|
| First-time setup | `node setup.cjs` |
| Vite dev server (frontend only) | `npm run dev` |
| Tauri dev (desktop + hot-reload) | `npm run tauri:dev` |
| Web server mode | `npm run web` or `node start.cjs --web` |
| Type-check | `npx vue-tsc --noEmit` |
| Build frontend | `npm run build` |
| Build full Tauri app | `npm run tauri:build` |
| Format code | `npm run format` |
| Check formatting | `npm run format:check` |
| Frontend unit tests | `npm run test:unit` |
| Rust unit tests | `cd src-tauri && cargo test` |
| Rust lint | `cd src-tauri && cargo clippy -- -D warnings` |
| Smoke test (PowerShell) | `.\scripts\smoke-test.ps1` |

Pre-commit hooks run Prettier automatically on staged files.

---

## Architecture

```
┌─────────────────────────────────────────────┐
│           Frontend (Vue 3 + TypeScript)       │
│  Explorer │ Reader │ Quiz/Diagnosis/ErrorBook │
│              Pinia Store                      │
└──────────────────┬──────────────────────────┘
                   │ Tauri IPC / HTTP Fetch
┌──────────────────▼──────────────────────────┐
│           Backend (Rust)                      │
│  note │ quiz │ settings │ mistake │ storage  │
│         Axum web server (web mode)           │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│           LLM (OpenAI-compatible API)        │
│         Ollama (default) or remote           │
└─────────────────────────────────────────────┘
```

- **Dual runtime**: The same Vue frontend runs inside Tauri **and** as a standalone web app served by Axum (`src-tauri/src/web_server.rs`).
- **Unified service layer**: All backend calls go through `src/services/`. Each service checks `isTauri()` and branches to `invoke()` (desktop) or `fetch()` (web).
- **Streaming**: Quiz generation and diagnosis use Tauri `Channel` on desktop and SSE on web, emitting the same event shapes.

See [AGENTS.md](AGENTS.md) for in-depth developer context.

---

## Data & Backup

### Where your data lives

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\knowtequiz\` |
| macOS | `~/Library/Application Support/knowtequiz/` |
| Linux | `~/.local/share/knowtequiz/` |

### Managed files

| File | Description |
|------|-------------|
| `settings.json` | LLM config, quiz defaults, workspace layout |
| `mistakes.jsonl` | Your mistake book (one JSON object per line) |
| `index.json` | Note metadata index (auto-generated, incremental) |
| `sessions/` | In-progress diagnosis sessions (auto-cleaned after 7 days) |
| `debug/` | Raw LLM responses for troubleshooting |
| `backups/` | Manual backups you create |

### Backup & Restore

- **Settings UI**: Open Settings → **Backup Data Now** / **Restore Latest Backup**
- **Open data directory**: Settings → **Open Folder**
- **Export mistakes**: Error Book → Export as JSON or Markdown

### Data Safety

- **Atomic writes**: Every file write uses `.tmp` → sync → rename pattern.
- **Automatic recovery**: Corrupt `settings.json` or `mistakes.jsonl` auto-recovers from `.bak`.
- **Auto-migration**: Legacy `mistakes.json` automatically migrates to `mistakes.jsonl` on first read.
- **Incremental index**: `index.json` reuses cached titles for unchanged files (size + modification time).

---

## LLM Configuration

KnowteQuiz connects to any **OpenAI-compatible** API endpoint.

| Setting | Default | Description |
|---------|---------|-------------|
| Base URL | `http://localhost:11434/v1` | Ollama local endpoint |
| Model | `qwen2.5:7b` | Any model available at your endpoint |
| API Key | *(empty)* | Required for remote endpoints |
| Temperature | `0.7` | Lower = more deterministic |
| Max Tokens | `4096` | Maximum response length |

Use **Test Connection** in Settings to verify your endpoint is reachable, and **Probe LLM Capabilities** to discover available models.

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+B` | Toggle left panel (Explorer) |
| `Ctrl+Shift+B` | Toggle right panel (Quiz/Diagnosis) |
| `Ctrl+F` | Search within current note |

---

## Testing

The project includes **66 Rust unit tests** and **152 frontend tests** covering:

- Atomic JSON writes and backup recovery
- Quiz JSON parsing and validation
- Answer normalization and scoring
- Note scanning, indexing, and frontmatter extraction
- Mistake filtering, search, and review flow
- SSE stream parsing
- Settings persistence

```bash
cd src-tauri && cargo test     # Rust tests
npm run test:unit              # Frontend tests
```

---

## License

[MIT](LICENSE)
