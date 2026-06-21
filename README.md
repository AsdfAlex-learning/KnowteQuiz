# KnowteQuiz

> A local-first Markdown knowledge quiz app with AI-powered blind-spot diagnosis.

**KnowteQuiz** turns your local Markdown notes into an interactive learning environment. It doesn't just ask questions—it diagnoses *why* you got them wrong through multi-turn Socratic dialogues, pinpointing exact knowledge gaps like "concept confusion" or "logical leaps."

---

## ✨ Features

- **Obsidian-grade reading** — MarkdownIt + KaTeX + syntax highlighting for immersive note reading.
- **Dual quiz modes**
  - **Basic Mode**: Quick self-test with instant scoring and mistake logging.
  - **Advanced Mode**: Submit your reasoning → AI asks follow-up questions → Receive a detailed blind-spot report.
- **Purely local** — All data (notes, mistakes, settings) stays on your machine as JSON files. Connect to any OpenAI-compatible endpoint (Ollama by default).
- **Three-pane workspace** — Explorer | Reader | Quiz & Diagnosis panel, with resizable columns and toggle shortcuts.
- **Cross-platform** — Built with Tauri v2 for Windows, macOS, and Linux.

---

## 🏗 Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop shell | Tauri v2 (Rust) |
| Frontend | Vue 3 + TypeScript + Vite + Pinia |
| Styling | TailwindCSS v4 |
| Markdown | MarkdownIt + KaTeX + highlight.js |
| Backend | Rust (Axum web server / Tauri dual-runtime) |
| Streaming | Tauri Channel / SSE |

---

## 🚀 Quick Start

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (Cargo)

### Install & Run

```bash
git clone https://github.com/yourname/knowtequiz.git
cd knowtequiz
npm install

# Development mode (Tauri with hot-reload)
node start.cjs --dev

# Or web-only mode
npm run web
```

### Build Release

```bash
npm run tauri:build
# Or use the convenience script
node start.cjs --build
```

---

## 🖥 Development

| Task | Command |
|------|---------|
| Vite dev server (frontend only) | `npm run dev` |
| Tauri dev (desktop + hot-reload) | `npm run tauri:dev` |
| Type-check | `npx vue-tsc --noEmit` |
| Build frontend | `npm run build` |
| Build full Tauri app | `npm run tauri:build` |
| Start release binary | `node start.cjs` |
| Web server mode | `npm run web` |
| Web + Desktop simultaneously | `npm run web:both` |
| Frontend unit tests | `npm run test:unit` |
| Rust unit tests | `cd src-tauri && cargo test` |

> **Note**: There is no linter or formatter configured yet.

---

## 🏛 Architecture Highlights

- **Dual runtime**: The same Vue frontend runs inside a Tauri desktop window **and** as a standalone web app served by an Axum server (`src-tauri/src/web_server.rs`).
- **Unified service layer**: All backend calls go through `src/services/`. Each service checks `isTauri()` and branches to `invoke()` (desktop) or `fetch()` (web).
- **Streaming symmetry**: Quiz generation and diagnosis use Tauri `Channel` on desktop and SSE on web, emitting the same event shapes.

See `AGENTS.md` for in-depth developer context.

---

## 📄 License

[MIT](LICENSE)
