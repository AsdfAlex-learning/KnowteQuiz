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
- **Custom appearance** — Six VS Code theme presets (Dark+ by default, Dark Modern, Light+, Monokai, Solarized Dark/Light) plus custom background/accent colours. The optional glassmorphism layer uses colourless frosted panels (5–12% tint, 40–60 px blur, saturate boost, directional edge light and film grain) over a night-scene background generated from the theme accent.
- **Dynamic backgrounds** — Use an MP4 as a looping background, with play / pause / stop controls. Stopping captures the current frame as a static wallpaper and releases the video from memory; resuming restarts from the saved position.
- **SM-2 spaced repetition** — Mistake book uses the SM-2 algorithm (Again / Hard / Good / Easy ratings) to schedule reviews at optimal intervals. Due items are surfaced with a "Due" filter and badge.
- **Daily streak tracking** — Consecutive study days are tracked and shown in the status bar.
- **Retry wrong answers** — After a quiz, re-practice only the questions you got wrong.
- **Markdown quiz content** — Quiz questions and diagnosis reports render full Markdown + KaTeX.
- **Document conversion** — Drop PDF / DOCX / PPTX / HTML files into your vault; they show up in the tree and convert to Markdown with one click. Text, headings, DOCX tables and embedded images are recovered, and the source file stays for lineage (a "stale" badge appears if you edit the original afterwards). Conversion is pure Rust — no Python, no external runtime.
- **Exploratory learning** — Pick notes and let the AI reorganise them into a shallow-to-deep knowledge map, then learn it through Socratic questions with a per-turn whiteboard. A 0–100 baseline slider tunes the map and question difficulty. Sessions auto-save and resume on re-entry (silently within 2 hours, otherwise via a restore prompt with full session history). Supports `/undo` `/redo` `/btw` `/skip` `/end`, Markdown learning-history export/import, and step-by-step replay.
- **Cross-platform** — Built with Tauri v2 for Windows, macOS, and Linux.
- **Dual runtime** — Same app runs as a Tauri desktop window **or** as a standalone web server (Axum).

---

## Quick Start

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (Cargo) — required for desktop mode only

### First-Time Setup

```bash
git clone https://github.com/AsdfAlex-learning/KnowteQuiz.git
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
| Lint (ESLint) | `npm run lint` |
| Frontend unit tests | `npm run test:unit` |
| Frontend test coverage | `npm run test:coverage` |
| Rust unit tests | `cd src-tauri && cargo test` |
| Rust lint | `cd src-tauri && cargo clippy -- -D warnings` |
| Rust format check | `cd src-tauri && cargo fmt --check` |
| Smoke test (PowerShell) | `.\scripts\smoke-test.ps1` |

Pre-commit hooks run Prettier + ESLint automatically on staged files. Commit messages are validated by commitlint (conventional commits).

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
| Windows | `%APPDATA%\com.knowtequiz.app\knowtequiz\` |
| macOS | `~/Library/Application Support/com.knowtequiz.app/knowtequiz/` |
| Linux | `~/.local/share/com.knowtequiz.app/knowtequiz/` |

### Managed files

| File | Description |
|------|-------------|
| `settings.json` | LLM config, quiz defaults, workspace layout |
| `mistakes.jsonl` | Your mistake book (one JSON object per line) |
| `index.json` | Note metadata index (auto-generated, incremental) |
| `sessions/` | In-progress diagnosis sessions (auto-cleaned after 7 days) |
| `media/` | Theme background images and videos |
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
| Base URL | *(empty)* | Required. Ollama example: `http://localhost:11434/v1` |
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

The project includes **194 Rust unit tests** and **289 frontend tests** (42 files) covering:

- Atomic JSON writes and backup recovery
- Quiz JSON parsing and validation
- Answer normalization and scoring
- Note scanning, indexing, and frontmatter extraction
- Mistake filtering, search, and SM-2 spaced repetition
- Concurrent mistake writes (no lost updates)
- Web path validation (reads bound to the configured note root)
- SSE stream parsing (multi-line and malformed events)
- Settings persistence and write queue serialization
- Theme color helpers, video-background controls, and glass blur clamping
- SM-2 golden vectors (shared between Rust and TypeScript)
- Exploration graph normalization, the turn state machine, undo/redo, and learning-history export → import round-trips

```bash
cd src-tauri && cargo test     # Rust tests
npm run test:unit              # Frontend tests
npm run test:coverage          # Frontend coverage report
```

---

## LLM Run Observability

Every LLM call is recorded so you can see what the model was asked and what it
returned. Open the **LLM** tab in the right-hand panel for a quick view: each
run shows its kind, duration, token usage, and status; expanding it reveals the
request parameters, the prompt, and the raw response with copy buttons.

External tools and custom pages can consume the same data through a read-only
API (web mode) or the on-disk JSONL file:

```bash
# Web mode: list the most recent runs
curl -s 'http://127.0.0.1:14200/api/llm/runs?limit=20' | jq

# Fetch the raw response of one run
curl -s 'http://127.0.0.1:14200/api/llm/runs/<id>/response'
```

```bash
# Every mode: greppable, tool-friendly JSONL
jq -c 'select(.error != null)' <app-data>/knowtequiz/llm_runs/llm_runs.jsonl
```

Browser-based tools require an origin allow-list in `settings.cors.allow_origins`
and a web-server restart; scripts and CLIs are unaffected. See
[`docs/llm-runs-schema.md`](docs/llm-runs-schema.md) for the full record schema,
endpoint reference, and integration examples.

Set `settings.llm.record_llm_io = false` to stop writing prompt/response bodies
while still recording timing and token usage. The API key is never recorded.

---

## License

[MIT](LICENSE)
