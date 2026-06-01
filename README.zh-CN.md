# KnowteQuiz

> 本地优先的 Markdown 知识测验应用，支持 AI 驱动的知识盲区深度诊断。

**KnowteQuiz** 将你的本地 Markdown 笔记转化为交互式学习环境。它不仅出题——更通过多轮苏格拉底式对话，诊断你「为什么答错」，精准定位「概念混淆」「逻辑跳跃」「审题失误」等知识盲区。

---

## ✨ 核心特性

- **Obsidian 级阅读体验** — MarkdownIt + KaTeX + 代码高亮，沉浸式阅读本地笔记。
- **双模式答题系统**
  - **基础模式**：快速自测，即时判分，记录错题。
  - **高级模式**：写出推理过程 → AI 多轮追问诊断 → 生成「知识盲区报告」。
- **纯本地运行** — 所有数据（笔记、错题、配置）均以 JSON 文件存储在本地。支持任意 OpenAI 兼容端点（默认 Ollama）。
- **三栏工作流** — 目录树 | 阅读区 | 出题/诊断/错题本面板，支持拖拽调整宽度与快捷键开关。
- **跨平台** — 基于 Tauri v2，支持 Windows、macOS 与 Linux。

---

## 🏗 技术栈

| 层级 | 技术 |
|------|------|
| 桌面端 | Tauri v2 (Rust) |
| 前端 | Vue 3 + TypeScript + Vite + Pinia |
| 样式 | TailwindCSS v4 |
| Markdown 渲染 | MarkdownIt + KaTeX + highlight.js |
| 后端 | Rust (Axum Web 服务器 / Tauri 双运行时) |
| 流式通信 | Tauri Channel / SSE |

---

## 🚀 快速开始

### 前置要求

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (Cargo)

### 安装与运行

```bash
git clone https://github.com/yourname/knowtequiz.git
cd knowtequiz
npm install

# 开发模式（Tauri + 热重载）
node start.cjs --dev

# 或仅启动 Web 模式
npm run web
```

### 构建发行版

```bash
npm run tauri:build
# 或使用便捷脚本
node start.cjs --build
```

---

## 🖥 开发命令

| 任务 | 命令 |
|------|------|
| Vite 开发服务器（仅前端） | `npm run dev` |
| Tauri 开发模式（桌面端 + 热重载） | `npm run tauri:dev` |
| 类型检查 | `npx vue-tsc --noEmit` |
| 构建前端 | `npm run build` |
| 构建完整 Tauri 应用 | `npm run tauri:build` |
| 启动发行版二进制文件 | `node start.cjs` |
| Web 服务器模式 | `npm run web` |
| Web + 桌面同时运行 | `npm run web:both` |

> **注意**：本项目尚未配置测试运行器、代码检查器或格式化工具。

---

## 🏛 架构亮点

- **双运行时**：同一套 Vue 前端既可在 Tauri 桌面窗口中运行，也可作为由 Axum 服务器独立托管的 Web 应用运行（`src-tauri/src/web_server.rs`）。
- **统一服务层**：所有后端调用均通过 `src/services/`。每个服务通过 `isTauri()` 判断当前环境，并分别调用 `invoke()`（桌面端）或 `fetch()`（Web 端）。
- **流式对称**：出题与诊断在桌面端使用 Tauri `Channel`，在 Web 端使用 SSE，两者 emit 完全相同的事件结构。

更深入的开发者上下文请见 `AGENTS.md`。

---

## 📄 许可证

[MIT](LICENSE)
