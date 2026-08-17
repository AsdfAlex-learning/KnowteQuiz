# KnowteQuiz

> 本地优先的 Markdown 知识测验应用，支持 AI 驱动的知识盲区深度诊断。

**KnowteQuiz** 将你的本地 Markdown 笔记转化为交互式学习环境。它不仅出题——更通过多轮苏格拉底式对话，诊断你「为什么答错」，精准定位「概念混淆」「逻辑跳跃」等知识盲区。

---

## 核心特性

- **Obsidian 级阅读体验** — MarkdownIt + KaTeX + 代码高亮，沉浸式阅读本地笔记。
- **双模式答题系统**
  - **基础模式**：快速自测，即时判分，记录错题。
  - **高级模式**：写出推理过程 → AI 多轮追问诊断 → 生成「知识盲区报告」。
- **纯本地运行** — 所有数据（笔记、错题、配置）均存储在本地。支持任意 OpenAI 兼容端点（默认 Ollama）。
- **三栏工作流** — 目录树 | 阅读区 | 出题/诊断/错题本面板，支持拖拽调整宽度与快捷键开关（`Ctrl+B` / `Ctrl+Shift+B`）。
- **跨平台** — 基于 Tauri v2，支持 Windows、macOS 与 Linux。
- **双运行时** — 同一应用既可作为 Tauri 桌面窗口运行，也可作为独立 Web 服务器运行（Axum）。

---

## 快速开始

### 前置要求

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (Cargo) — 仅桌面模式需要

### 首次安装

```bash
git clone https://github.com/yourname/knowtequiz.git
cd knowtequiz
node setup.cjs          # 交互式安装：检查环境 → 安装依赖 → 构建 → 选择模式
```

### 运行

```bash
node start.cjs              # 桌面应用（未构建则自动构建）
node start.cjs --web        # Web 服务，浏览器访问 http://localhost:14200
node start.cjs --dev        # 开发模式，支持热重载
node start.cjs --both       # 桌面 + Web 同时启动
```

### 构建发行版

```bash
node start.cjs --build      # 或：npm run tauri:build
```

---

## 开发命令

| 任务 | 命令 |
|------|------|
| 首次安装 | `node setup.cjs` |
| Vite 开发服务器（仅前端） | `npm run dev` |
| Tauri 开发模式（桌面端 + 热重载） | `npm run tauri:dev` |
| Web 服务器模式 | `npm run web` 或 `node start.cjs --web` |
| 类型检查 | `npx vue-tsc --noEmit` |
| 构建前端 | `npm run build` |
| 构建完整 Tauri 应用 | `npm run tauri:build` |
| 格式化代码 | `npm run format` |
| 检查格式 | `npm run format:check` |
| 前端单元测试 | `npm run test:unit` |
| Rust 单元测试 | `cd src-tauri && cargo test` |
| Rust 代码检查 | `cd src-tauri && cargo clippy -- -D warnings` |
| 冒烟测试（PowerShell） | `.\scripts\smoke-test.ps1` |

提交代码时 Prettier 会自动格式化暂存文件。

---

## 架构概览

```
┌─────────────────────────────────────────────┐
│           前端 (Vue 3 + TypeScript)           │
│  目录树 │ 阅读区 │ 出题/诊断/错题本           │
│              Pinia Store                      │
└──────────────────┬──────────────────────────┘
                   │ Tauri IPC / HTTP Fetch
┌──────────────────▼──────────────────────────┐
│           后端 (Rust)                         │
│  note │ quiz │ settings │ mistake │ storage  │
│         Axum Web 服务器 (Web 模式)            │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│           LLM (OpenAI 兼容 API)              │
│         Ollama (默认) 或远程端点              │
└─────────────────────────────────────────────┘
```

- **双运行时**：同一套 Vue 前端既可在 Tauri 桌面窗口中运行，也可作为由 Axum 服务器独立托管的 Web 应用运行。
- **统一服务层**：所有后端调用均通过 `src/services/`，通过 `isTauri()` 判断环境并分支。
- **流式通信**：出题与诊断在桌面端使用 Tauri `Channel`，在 Web 端使用 SSE，事件结构完全一致。

更深入的开发者上下文请见 [AGENTS.md](AGENTS.md)。

---

## 数据与备份

### 数据存储位置

| 平台 | 路径 |
|------|------|
| Windows | `%APPDATA%\knowtequiz\` |
| macOS | `~/Library/Application Support/knowtequiz/` |
| Linux | `~/.local/share/knowtequiz/` |

### 托管文件

| 文件 | 说明 |
|------|------|
| `settings.json` | LLM 配置、答题默认值、工作区布局 |
| `mistakes.jsonl` | 错题本（每行一条 JSON 对象） |
| `index.json` | 笔记元数据索引（自动生成，增量更新） |
| `sessions/` | 进行中的诊断会话（7 天后自动清理） |
| `debug/` | LLM 原始响应日志（用于调试） |
| `backups/` | 手动创建的备份 |

### 备份与恢复

- **设置界面**：打开设置 → **立即备份** / **恢复最新备份**
- **打开数据目录**：设置 → **打开文件夹**
- **导出错题**：错题本 → 导出为 JSON 或 Markdown

### 数据安全

- **原子写入**：每次文件写入使用 `.tmp` → sync → rename 模式。
- **自动恢复**：`settings.json` 或 `mistakes.jsonl` 损坏时自动从 `.bak` 恢复。
- **自动迁移**：首次读取时自动将旧版 `mistakes.json` 迁移为 `mistakes.jsonl`。
- **增量索引**：`index.json` 对未变化文件复用缓存标题（基于文件大小 + 修改时间）。

---

## LLM 配置

KnowteQuiz 支持任意 **OpenAI 兼容** API 端点。

| 设置项 | 默认值 | 说明 |
|--------|--------|------|
| Base URL | `http://localhost:11434/v1` | Ollama 本地端点 |
| Model | `qwen2.5:7b` | 端点上可用的任意模型 |
| API Key | *（空）* | 远程端点需要 |
| Temperature | `0.7` | 越低越确定性 |
| Max Tokens | `4096` | 最大响应长度 |

在设置中使用 **测试连接** 验证端点可达性，使用 **探测 LLM 能力** 发现可用模型。

---

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+B` | 切换左侧面板（目录树） |
| `Ctrl+Shift+B` | 切换右侧面板（出题/诊断） |
| `Ctrl+F` | 在当前笔记中搜索 |

---

## 测试

项目包含 **66 个 Rust 单元测试**和 **152 个前端测试**，覆盖：

- 原子 JSON 写入与备份恢复
- Quiz JSON 解析与校验
- 答案归一化与评分
- 笔记扫描、索引与 frontmatter 提取
- 错题过滤、搜索与复习流程
- SSE 流解析
- 设置持久化

```bash
cd src-tauri && cargo test     # Rust 测试
npm run test:unit              # 前端测试
```

---

## 许可证

[MIT](LICENSE)
