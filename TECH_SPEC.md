# KnowteQuiz 技术方案文档（V1.0 正式版）

> **状态：历史设计稿（V1.0 蓝图）**。本文档记录项目立项时的架构设想，部分内容（命令表、目录树、测试计数、V1.1 路线图）已随实现演进过时。
>
> **以实际代码与最新文档为准**：
> - `README.md` / `README.zh-CN.md` — 当前功能与用法
> - `AGENTS.md` — 智能体开发上下文
> - `docs/project-review-2026-10.md` + `docs/review-2026-10-09.md` — 近期审查与修复
> - `docs/optimization-plan.md` — 进度追踪
>
> 当前实际：36 前端测试文件 / 227 用例 + 146 Rust 测试；`doc_parser`（纯 Rust，PDF/DOCX/PPTX/HTML→Markdown）**已实现**，`ocr_engine.rs`（扫描件升级项）未实现；无 `views/`、无 `tailwind.config.js`。

> 基于 Grill-Me 决策会话收敛的终版方案。本文档替代 `idea.md` 作为项目开发的核心依据。

---

## 1. 项目总览

### 1.1 定位

一款**本地优先、极致轻量**的 Markdown 知识自测桌面应用。以本地 Markdown 笔记为唯一数据源，集成 LLM API 实现智能出题与弱项分析，帮助用户在阅读笔记的过程中即时检验和巩固知识。

### 1.2 核心原则

1. **纯本地运行**：所有数据（笔记、错题、配置）均存储在本地，无云端同步，无数据上传。
2. **极致轻量**：安装包体积小，资源占用低，启动速度快。
3. **阅读优先**：核心体验对标 Obsidian 阅读模式，专注、沉浸、渲染质量高。
4. **可选答题**：答题是阅读后的可选动作，非强制流程，不引入复杂的进度追踪或学习统计系统。
5. **解耦设计**：核心模块（笔记管理、出题引擎、LLM 调用、渲染）保持低耦合，便于迭代。

### 1.3 V1.0 MVP 范围

**包含：**
- 本地文件夹指定与 Markdown 目录树浏览
- Obsidian 风格 Markdown 阅读渲染（含 LaTeX 数学公式）
- 基础实时出题：选中笔记 → 调用 LLM API 生成题目 → 答题 → 记录错题
- 简单错题本（JSON 存储，仅记录错题原文与用户答案）
- 软件基础配置（主题、LLM API 地址、模型选择等）
- 三栏布局（目录树 | 阅读区 | 功能面板），**支持灵活开关各栏**

**明确不包含（后续版本规划）：**
- PDF / Word / PPT / HTML 转 Markdown（V1.1，**纯 Rust 实现**）
- OCR 扫描件识别（V1.1，Rust + ONNX，与转换管线可组合）
- 用户自定义提示词模板引擎（V1.1）
- 错题复习模式（V1.1）
- 标签系统、学习统计、进度追踪（V1.2 或永不）

---

## 2. 技术栈

| 层级 | 技术选型 | 说明 |
|---|---|---|
| **桌面壳** | **Tauri v2** | 轻量、安全、跨平台（Win/macOS/Linux），Rust 后端 |
| **前端框架** | **Vue 3 + TypeScript** | 组合式 API，响应式系统，类型安全 |
| **构建工具** | **Vite** | Tauri 官方推荐，极速 HMR |
| **状态管理** | **Pinia** | Vue 官方推荐，TypeScript 友好 |
| **UI 组件库** | **待定**（建议 Headless UI + Tailwind CSS） | 保持样式自由度，避免组件库体积过大 |
| **Markdown 阅读** | **MarkdownIt + KaTeX + 自定义 CSS** | 对标 Obsidian 阅读体验，支持 LaTeX |
| **Markdown 编辑** | **Milkdown / Vditor（可选嵌入）** | 作为辅助功能，非 V1.0 核心 |
| **数据存储** | **纯 JSON 文件** | 应用配置目录下存储，人类可读，便于迁移 |
| **文件系统** | **Tauri FS + Path API** | 前端通过 Tauri Command 调用 Rust 后端进行文件 IO |
| **LLM 调用** | **Rust 后端 HTTP Client** | 兼容 OpenAI API 格式，用户配置 endpoint |
| **进程通信** | **Tauri Command / Event** | 前端 `invoke` 调用 Rust 函数，Rust `emit` 推送事件 |
| **OCR（V1.1）** | **Rust + ONNX Runtime (`ort` crate)** | 默认加载 RapidOCR ONNX 模型，可配置其他引擎 |
| **文档解析（V1.1）** | **纯 Rust（无外部运行时）** | OOXML（DOCX/PPTX）走 `zip` + `quick-xml` 直接解析；PDF 走 `pdf-extract` / `lopdf` 纯文本抽取。零安装依赖，符合"纯本地、极致轻量"定位；PPT 与 Word 一并覆盖。**表格/版式高保真与扫描件 OCR 为独立升级项**（见附录 A 决策 2）。 |

---

## 3. 系统架构

### 3.1 架构分层

```
┌─────────────────────────────────────────────┐
│              前端交互层 (Vue 3)                │
│  ┌─────────┐  ┌──────────┐  ┌────────────┐ │
│  │ 目录树   │  │ 阅读渲染  │  │ 功能面板   │ │
│  │ (左栏)   │  │ (中栏)   │  │ (右栏)    │ │
│  └────┬────┘  └────┬─────┘  └─────┬──────┘ │
│       └─────────────┴──────────────┘        │
│              Pinia Store (状态)              │
└──────────────────┬──────────────────────────┘
                   │ Tauri IPC
┌──────────────────▼──────────────────────────┐
│           应用服务层 (Tauri Rust)              │
│  ┌──────────┐ ┌──────────┐ ┌──────────────┐ │
│  │ 笔记管理  │ │ 出题引擎  │ │ 配置与数据   │ │
│  │  Command │ │  Command │ │  Service     │ │
│  └──────────┘ └──────────┘ └──────────────┘ │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│           底层依赖层                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────────┐ │
│  │ 文件系统  │ │ HTTP Client│ │ JSON 存储    │ │
│  │ (fs插件) │ │ (reqwest)│ │ (serde_json) │ │
│  └──────────┘ └──────────┘ └──────────────┘ │
└──────────────────┬──────────────────────────┘
                   │ 同进程调用 (V1.1)
┌──────────────────▼──────────────────────────┐
│         文档解析引擎层 (Rust，无外部运行时)      │
│  ┌──────────┐ ┌──────────┐ ┌──────────────┐ │
│  │ OOXML    │ │ PDF      │ │ HTML         │ │
│  │ zip+quick│ │ pdf-extract│ │ 复用 MarkdownIt │ │
│  │ -xml     │ │ /lopdf   │ │              │ │
│  │(DOCX/PPT)│ │          │ │              │ │
│  └──────────┘ └──────────┘ └──────────────┘ │
│           （扫描件 OCR 走 ort，可叠加）        │
└─────────────────────────────────────────────┘
```

### 3.2 模块划分与职责

#### 前端模块 (Vue 3)

| 模块 | 职责 | 对应 UI 区域 |
|---|---|---|
| `Explorer` | 目录树渲染、文件夹/文件浏览、文件监听响应 | 左栏 |
| `Reader` | Markdown 渲染、LaTeX 解析、滚动位置记忆 | 中栏 |
| `Panel` | 动态功能面板容器（出题、错题、设置） | 右栏 |
| `QuizGenerator` | 出题参数表单、模板选择、进度展示 | 右栏 - 出题 |
| `QuizSession` | 题目展示、答题交互、提交判分 | 右栏 - 答题中 |
| `QuizResult` | 答题结果、错题标记、弱项提示 | 右栏 - 答题后 |
| `ErrorBook` | 错题列表、查看原文 | 右栏 - 错题本 |
| `Settings` | 全局配置（LLM、主题、快捷键） | 右栏 - 设置 |
| `Exploration` | 探索性学习：源笔记多选、知识脉络、苏格拉底式问答、白板、回放 | 探索模式（三栏复用） |
| `LayoutManager` | 三栏显隐控制、宽度调整、状态持久化 | 全局 |

#### 后端模块 (Rust)

| 模块 | 职责 | 核心 crate |
|---|---|---|
| `fs_service` | 递归扫描笔记目录、读取文件、监听变更 | `tokio::fs`, `notify` |
| `note_service` | 笔记元数据提取（标题、 frontmatter） | `regex`, `yaml-rust` |
| `quiz_engine` | 构建提示词、调用 LLM API、解析返回题目 | `reqwest`, `serde_json` |
| `storage` | JSON 文件的读写、数据校验、版本迁移 | `serde`, `serde_json` |
| `config` | 配置管理、默认值、热重载 | `confy` |
| `ocr_engine` (V1.1) | 图像预处理、ONNX 推理、后处理；供扫描件 PDF 升级路径复用 | `ort`, `image` |
| `doc_parser` (V1.1) | 文档格式分派（OOXML/PDF/HTML）→ 结构化 Markdown；含进度事件与错误归一化 | `zip`, `quick-xml`, `pdf-extract`, `serde_json` |
| `exploration` (V1.2) | 探索会话持久化、map-reduce 知识图谱、轮次引擎、旁支问答、学习史导出/导入 | `serde_json`, `uuid`, `chrono` |

**格式解析子模块（V1.1，均在 `doc_parser` 内）**：

| 子模块 | 输入格式 | 解析方式 |
|---|---|---|
| `ooxml` | `.docx` / `.pptx` | 解压 → 取 `word/document.xml` 或 `ppt/slides/slide*.xml` → 提取 `w:t` / `a:t` 文本节点，按段落/列表层级还原 |
| `pdf` | `.pdf` | 文本层抽取（`pdf-extract`），保留阅读顺序与分页标记 |
| `html` | `.html` / `.htm` | 复用已有的 MarkdownIt 管线做 HTML→MD 清洗 |

---

## 4. 数据存储设计

### 4.1 存储策略

**纯 JSON 文件**，存储于 Tauri 应用配置目录（`app_data_dir`）下的 `knowtequiz/` 文件夹中。

**设计理由**：
- 数据结构简单，无复杂关联查询需求。
- JSON 人类可读，便于调试、备份和迁移。
- 避免引入 SQLite 等额外依赖，保持轻量。

### 4.2 文件清单

| 文件名 | 内容 | 结构 |
|---|---|---|
| `settings.json` | 用户全局配置 | 扁平对象 |
| `mistakes.jsonl` | 错题记录（每行一条 JSON 对象） | NDJSON 行 |
| `index.json` | 笔记元数据索引（增量更新） | 对象（`root_path` + `notes[]` 元数据） |
| `sessions/` | 进行中的诊断会话 | JSON 文件 |
| `exploration/sessions/` | 探索性学习会话（知识图谱、时间线、白板、旁支） | JSON 文件 |
| `exploration/exports/` | 自动导出的学习史 Markdown | Markdown 文件 |
| `llm_runs/` | LLM 调用记录（元数据 JSONL；`debug/` 存原始请求/响应） | NDJSON + 文本 |
| `debug/` | LLM 原始响应日志 | JSON 文件 |
| `backups/` | 手动备份 | 时间戳目录 |

### 4.3 数据 Schema

#### `settings.json`

```json
{
  "version": "1.0.0",
  "theme": "obsidian-dark",
  "llm": {
    "provider": "openai-compatible",
    "base_url": "http://localhost:11434/v1",
    "api_key": "",
    "model": "qwen2.5:7b",
    "max_tokens": 4096,
    "temperature": 0.7
  },
  "ocr": {
    "engine": "rapidocr",
    "model_path": "",
    "fallback_engines": ["tesseract"]
  },
  "ui": {
    "layout": {
      "left_visible": true,
      "right_visible": true,
      "left_width": 280,
      "right_width": 360
    }
  },
  "quiz": {
    "default_types": ["single", "multiple", "short"],
    "default_language": "zh",
    "default_count": 5,
    "default_mode": "basic",
    "advanced": {
      "max_diagnosis_rounds": 3,
      "require_reasoning": true,
      "show_diagnosis_report": true
    }
  }
}
```

#### `mistakes.json`

支持两种模式的错题记录。低级模式条目精简；高级模式条目包含完整的诊断上下文。

```json
[
  {
    "id": "uuid",
    "note_path": "/notes/rust/ownership.md",
    "note_title": "Rust Ownership",
    "question": "以下关于 Ownership 的说法，错误的是？",
    "user_answer": "A",
    "correct_answer": "B",
    "explanation": "...",
    "mode": "basic",
    "created_at": "2026-05-29T10:00:00Z",
    "review_count": 0
  },
  {
    "id": "uuid",
    "note_path": "/notes/calculus/series.md",
    "note_title": "数列与级数",
    "question": "以下关于数列收敛性的说法，错误的是？",
    "user_answer": "B",
    "correct_answer": "C",
    "user_reasoning": "我选B是因为我记得所有收敛数列都必然有界...",
    "mode": "advanced",
    "diagnosis": {
      "rounds": 2,
      "conversation": [
        {
          "role": "ai",
          "content": "你提到单调有界数列一定收敛，这是对的...",
          "blind_spots": [{"type": "逻辑跳跃", "concept": "题意理解"}],
          "follow_up": "请再仔细看题目要求..."
        },
        {
          "role": "user",
          "content": "哦我看错了..."
        }
      ],
      "final_report": {
        "summary": "你的主要问题在于概念混淆和审题...",
        "blind_spots": [
          {
            "tag": "概念混淆：收敛 vs 有界",
            "severity": "高",
            "description": "...",
            "note_reference": "笔记第3段：'...'",
            "suggestion": "..."
          }
        ],
        "overall_level": "基础概念理解不牢，需回溯定义",
        "next_steps": ["..."]
      }
    },
    "created_at": "2026-05-29T10:00:00Z",
    "review_count": 0
  }
]
```

---

## 5. 核心业务流程

### 5.1 笔记阅读流程

```
用户打开应用
  → 首次使用：引导选择本地笔记文件夹
  → 启动时全量扫描该目录下所有 .md 文件
  → 构建目录树（左栏）
  → 用户点击某笔记
  → Rust 后端读取文件内容
  → 返回前端渲染为 HTML（MarkdownIt + KaTeX）
  → 中栏展示阅读内容
```

**关键点**：
- 不维护持久化索引，每次启动全量扫描。目录较大时的延迟需在 UX 上处理（如异步加载、骨架屏）。
- 文件变更通过 Tauri 的 `fs` API 实时读取，不监听文件系统事件（V1.0 简化）。

### 5.2 实时出题流程

```
用户在阅读区选中某笔记
  → 点击右栏"出题"面板
  → 前端传递参数：笔记路径、题型、数量、难度、语言
  → Rust 后端读取笔记全文
  → 拼接固定提示词模板（内置 1-2 个）
  → 调用用户配置的 LLM API
  → 流式返回题目 JSON
  → 前端实时展示题目
  → 用户答题并提交
  → 前端对比标准答案（选择题）或展示参考答案（简答题）
  → 标记错题 → 写入 mistakes.json
```

**内置提示词模板（V1.0）**：

1. **基础出题模板**：基于笔记全文，生成指定数量、指定题型、指定难度的题目。
2. **错题分析模板**：用户提交错题后，调用 LLM 分析该题对应的知识点盲区，并引用原文内容解释。

### 5.3 错题本流程

```
用户点击右栏"错题本"
  → 读取 mistakes.json
  → 按时间倒序展示
  → 点击某错题 → 打开对应笔记并定位到相关段落（V1.1：简单跳转，V1.0 可能仅打开笔记）
```

### 5.4 文档解析流程（V1.1）

```
用户在笔记目录中选中一个非 MD 文件（PDF / DOCX / PPTX / HTML）
  → 右键点击"转换为 Markdown"
  → Rust 后端 `doc_parser` 模块接收任务
  → 按扩展名分派解析子模块（ooxml / pdf / html）
      OOXML：解压 zip → 解析 document.xml / slide*.xml → 提取文本节点
      PDF：读取文本层 → 保留阅读顺序与分页
      HTML：复用 MarkdownIt 管线清洗转换
  → （可选升级路径）若 PDF 无文本层，交给 ocr_engine 做 ONNX 识别
  → 结果归一为结构化 Markdown（含元数据与错误信息）
  → 将生成的 Markdown 文件保存到笔记目录（用户可自定义目标路径）
  → 通知前端刷新目录树，用户可在笔记库中看到新生成的 .md 文件
  → 用户可立即阅读、出题
```

**关键技术细节**：
- **零外部运行时**：解析全部在 Rust 进程内完成，不 spawn 子进程、不依赖用户安装 Python。
- **统一输出契约**：无论哪种格式，`doc_parser` 都返回同一个 `ParseResult`（Markdown 文本 + 元数据 + 错误信息），前端不感知底层引擎。
- **并发处理**：解析在 `tokio::spawn_blocking` 中执行（zip/XML/PDF 解析是 CPU 密集的同步工作），不阻塞异步运行时；前端显示进度，不卡 UI。
- **错误处理**：损坏文件、加密文档、无文本层 PDF 都映射为结构化 `AppError`，包含文件名与原因，前端展示可读提示。
- **OCR 升级空间**：`ocr_engine` 与 `doc_parser` 解耦 —— PDF 解析先走文本层，检测到"无文本层 / 文本极少"时可触发 OCR 分支。OCR 模型仍按需下载（见 9. 风险表），不进安装包。
- **格式白名单**：`doc_parser` 只接受 `.pdf` / `.docx` / `.pptx` / `.html` / `.htm` 扩展名，其余拒绝，防止被当作任意文件读取入口。

### 5.5 答题交互设计（核心创新模块）

#### 5.4.1 设计目标

答题模块不只是一个"做题对错判断"的工具，而是一个**知识盲区诊断器**。它模仿苏格拉底式提问，通过分析用户的推理过程，精确定位用户"到底哪里不会"。

交互体验对标 **Opencode Question 工具**：卡片式选项、清晰的层次结构、流畅的多轮对话。

#### 5.4.2 双模式架构

右栏出题面板提供**模式切换开关**：

| 模式 | 定位 | 适合场景 |
|---|---|---|
| **低级模式 (Basic Mode)** | 快速自测，验证知识记忆 | 时间紧、快速过一遍、确认基础掌握度 |
| **高级模式 (Advanced Mode)** | 深度诊断，定位知识盲区 | 想真正理解知识、找出思维漏洞、针对性补强 |

**模式切换时机**：在出题前选择，题目生成阶段不受模式影响；答题阶段根据模式展示不同的交互界面。

#### 5.4.3 低级模式 (Basic Mode) 交互流程

```
右栏展示题目卡片
  → 用户阅读题干和选项（单选/多选/简答）
  → 对于选择题：点击选项卡片选中（类似 Opencode 的选项卡片样式）
  → 对于简答题：在文本框中输入答案
  → 点击"提交"按钮
  → 前端即时判分：
      - 选择题：对比标准答案，绿色正确 / 红色错误
      - 简答题：展示参考答案，由用户自行判断是否掌握
  → 若答错：
      - 展示正确答案和解析
      - 提供"记录错题"按钮（一键写入 mistakes.json）
      - 提供"深入诊断"按钮（切换到高级模式继续分析）
  → 进入下一题或结束答题
```

**UI 设计要点**：
- **题目卡片**：白色/深色卡片背景，题干用正文渲染，支持 LaTeX。
- **选项卡片**：垂直排列，每项一个卡片，包含选项字母 + 文本描述。选中态用主色边框高亮，错误态红色，正确态绿色。
- **操作栏**：底部固定"提交"按钮，提交后变为"下一题" / "查看解析" / "结束"。
- **即时反馈**：无需调用 LLM，前端本地判分，零延迟。

#### 5.4.4 高级模式 (Advanced Mode) 交互流程 — 知识盲区诊断

高级模式是 KnowteQuiz 的核心差异化功能。它不关注"你选错了什么"，而是关注"**你为什么这么想**"。

**整体流程**：

```
右栏展示题目卡片（与低级模式相同）
  → 用户选择/输入答案
  → 【新增】用户必须在理由输入框中写出自己的推理过程
      - 例如："我选 B 是因为我记得收敛数列必须有界，而题目中数列..."
  → 点击"提交并诊断"
  → Rust 后端将【题目 + 用户答案 + 用户理由 + 笔记原文】一并发送给 LLM
  → LLM 运行诊断分析（见 5.4.5 诊断提示词设计）
  → 前端展示诊断对话流（类似聊天界面）：
      - 第一轮：AI 给出初步判断（"你的推理在第二步有漏洞..."）
      - AI 提出一个追问问题，针对具体盲区
      - 用户在输入框中回答追问
      - 循环 2-3 轮（可配置）
  → 最终生成【诊断报告】：
      - 知识盲区标签（如"概念混淆：收敛 vs 有界" / "计算错误：极限运算" / "理解缺失：柯西准则"）
      - 对应笔记原文中的相关段落引用
      - 针对性学习建议
  → 用户可一键保存错题（包含完整诊断对话和报告）
```

**UI 设计要点**：

1. **答题阶段**：与低级模式相同的题目卡片 + 选项卡片。
2. **理由输入区**：
   - 在选项下方展开一个文本区域，提示文字："请写出你的推理过程，越详细越好。例如：你为什么这样选？你用了什么公式/定理？"
   - 支持多行输入，最小高度 3 行，可拖拽扩展。
   - 提供"不太确定"快捷按钮，自动生成理由占位符。
3. **诊断对话流**：
   - 提交后，界面切换为**聊天式对话流**。
   - AI 消息气泡：白色/浅色，包含诊断分析和追问问题。
   - 用户消息气泡：主色/深色，展示自己的回答。
   - 每条 AI 消息后附带一个输入框（单轮回复）。
   - 提供"结束诊断"按钮，随时可提前终止并查看当前诊断报告。
4. **诊断报告卡片**：
   - 最终展示一个汇总卡片，包含：
     - **盲区标签**（Chip 形式，可点击跳转到笔记对应段落）
     - **推理分析**（AI 对用户推理过程的逐段点评）
     - **原文引用**（从笔记中定位的相关内容）
     - **建议**（下一步该学什么）
   - 底部提供"保存到错题本"和"重新诊断"按钮。

#### 5.4.5 诊断提示词设计（高级模式 Rust 后端）

高级模式需要 3 个核心提示词模板：

**模板 A：初步诊断 (Initial Diagnosis)**

```
你是一位严格但友善的数学/计算机科学/XX 学科导师。

【笔记原文】
{{note_content}}

【题目】
{{question}}

【标准答案】
{{correct_answer}}

【用户答案】
{{user_answer}}

【用户推理过程】
{{user_reasoning}}

请按以下步骤分析：
1. 判断用户的答案是否正确。
2. 仔细阅读用户的推理过程，逐句分析其逻辑链条中的每个环节。
3. 找出推理中的第一个错误环节，并判断错误的类型：
   - 概念理解错误（对某个定义/定理的理解有误）
   - 知识盲区（完全不知道某个概念/定理）
   - 逻辑跳跃（跳过了关键步骤）
   - 计算/应用错误（知道概念但用错了）
   - 混淆（把 A 概念和 B 概念搞混了）
4. 基于第一个错误环节，提出一个针对性的追问问题，引导用户暴露更深层的理解状况。

以 JSON 格式返回：
{
  "is_correct": false,
  "answer_analysis": "用户答案错误。正确答案是 B...",
  "reasoning_review": [
    { "step": "用户说：'...'", "assessment": "正确", "comment": "..." },
    { "step": "用户说：'...'", "assessment": "错误", "comment": "...", "error_type": "概念理解错误", "related_concept": "有界数列" }
  ],
  "blind_spot": {
    "type": "概念理解错误",
    "concept": "收敛数列与有界数列的关系",
    "description": "用户混淆了..."
  },
  "follow_up_question": "你提到有界数列一定收敛，那你能举一个反例吗？比如 (-1)^n？"
}
```

**模板 B：追问诊断 (Follow-up Diagnosis)**

```
【上下文】我们正在诊断用户对"{{topic}}"的掌握情况。

【前序诊断】
{{previous_diagnosis_json}}

【用户对上轮追问的回答】
{{user_follow_up_answer}}

请分析用户的这个回答，判断：
1. 用户的理解是否有所修正？
2. 是否暴露了新的知识盲区？
3. 是否需要继续追问？如果需要，请提出新的追问问题。

以 JSON 返回：
{
  "progress_assessment": "用户开始意识到...但仍未完全理解...",
  "new_blind_spots": [{"type": "...", "concept": "..."}],
  "should_continue": true,
  "follow_up_question": "...",
  "is_final": false
}
```

**模板 C：诊断报告生成 (Final Report)**

```
【完整诊断对话记录】
{{diagnosis_conversation_json}}

【笔记原文】
{{note_content}}

请生成最终的诊断报告，要求：
1. 用中文输出，语气友善且专业。
2. 列出所有发现的知识盲区（Chip 标签形式）。
3. 引用笔记原文中的相关段落，指出用户应该重点回顾的内容。
4. 给出具体的学习建议（不是泛泛而谈）。

以 JSON 返回：
{
  "summary": "你在数列收敛性的理解上，主要存在两个层面的问题...",
  "blind_spots": [
    {
      "tag": "概念混淆：收敛 vs 有界",
      "severity": "高",
      "description": "你认为有界一定收敛，但实际上...",
      "note_reference": "笔记第3段：'收敛数列必有界，反之不成立...'",
      "suggestion": "重点复习"收敛"与"有界"的定义差异，做5道反例判断题。"
    }
  ],
  "overall_level": "基础概念薄弱，需回溯定义",
  "next_steps": ["重新阅读笔记中关于柯西收敛准则的部分", "练习证明题：证明收敛数列必有界"]
}
```

#### 5.4.6 高级模式状态机

```
[Idle] --用户点击"开始答题"--> [Answering]

[Answering] --提交答案+理由--> [Diagnosing_Round_1]
  └-- 可编辑答案/理由，重新提交

[Diagnosing_Round_1] --用户回复追问--> [Diagnosing_Round_2]
  └-- 点击"结束诊断" --> [Report]

[Diagnosing_Round_2] --用户回复追问--> [Diagnosing_Round_3]
  └-- 点击"结束诊断" --> [Report]

[Diagnosing_Round_3] --自动完成或用户结束--> [Report]

[Report] --保存错题--> [Idle]
  └--重新诊断--> [Answering]（清空理由，重新输入）
```

最大诊断轮次可配置（默认 3 轮），避免 LLM 调用成本过高。

#### 5.4.7 数据结构设计扩展

高级模式需要扩展 `mistakes.json` 以保存完整的诊断上下文：

**`mistakes.json` 高级模式条目**：

```json
{
  "id": "uuid",
  "note_path": "/notes/calculus/series.md",
  "note_title": "数列与级数",
  "question": "以下关于数列收敛性的说法，错误的是？",
  "user_answer": "B",
  "correct_answer": "C",
  "user_reasoning": "我选B是因为我记得所有收敛数列都必然有界，而单调有界数列一定收敛，所以这两个条件合在一起...",
  "mode": "advanced",
  "diagnosis": {
    "rounds": 2,
    "conversation": [
      {
        "role": "ai",
        "content": "你提到单调有界数列一定收敛，这是对的。但题目问的是'错误的说法'，你选择B意味着你认为B是错误的...",
        "blind_spots": [{"type": "逻辑跳跃", "concept": "题意理解"}],
        "follow_up": "请再仔细看题目要求：题目要你选出'错误的说法'，你确定B是错误说法吗？"
      },
      {
        "role": "user",
        "content": "哦我看错了，我以为B说'所有有界数列都收敛'，原来B说的是'所有收敛数列都有界'..."
      },
      {
        "role": "ai",
        "content": "很好，你发现了第一个问题：审题不仔细。但更深层的问题是...",
        "blind_spots": [{"type": "概念混淆", "concept": "收敛 vs 有界"}],
        "follow_up": "那你能举一个'有界但不收敛'的反例吗？"
      }
    ],
    "final_report": {
      "summary": "你的主要问题在于概念混淆和审题...",
      "blind_spots": [
        {
          "tag": "概念混淆：收敛 vs 有界",
          "severity": "高",
          "description": "你潜意识里认为有界和收敛是等价的...",
          "note_reference": "笔记第3段：'收敛数列必有界，反之不成立...'",
          "suggestion": "重点复习定义差异，构造反例。"
        },
        {
          "tag": "审题失误",
          "severity": "中",
          "description": "选择了正确的说法而非错误说法。",
          "suggestion": "做题时圈出题目中的'错误'、'不正确'等关键词。"
        }
      ],
      "overall_level": "基础概念理解不牢，需回溯定义",
      "next_steps": ["复习收敛/有界/单调的定义", "练习构造反例"]
    }
  },
  "created_at": "2026-05-29T10:00:00Z",
  "review_count": 0
}
```

---

### 5.6 探索性学习模式（V1.2，已完成）

**定位**：与「阅读 + 出题」并列的第二种学习方式。选中若干 Markdown 笔记后，由 LLM 跨文档重组出一条由浅入深的知识脉络，再以苏格拉底式提问逐点带学；白板按轮次版本化，学习史可导出、导入与回放。完整设计见 `docs/exploration-mode-plan.md`。

**入口与布局**：`Explorer` 底部模式按钮；`settings.ui.last_mode` 持久化。探索模式下左栏复用 `Explorer`（多选复选框 + 全选/清除），中栏换成 `ExplorationShell` / `ExplorationChat`，右栏换成 `ExplorationPanel`（状态 + 白板）。列宽与折叠复用既有布局，`TitleBar` 不变。

**状态机**：`Initializing → (AwaitingDomainConfirm) → Active ⇄ /undo ⇄ /redo → Completed`。导入的会话以只读 `Completed` 打开，可「从此继续」重新激活。

**初始化（map-reduce）**：

1. 逐篇读取源笔记（`fs_service::resolve_note_path` 绑定笔记根），截断后生成结构化摘要（`exploration_summary`）。
2. 汇总所有摘要，一次生成跨文档图谱（`exploration_graph`）：浅 → 深、至少一条跨文档重组边、每个概念带 `source_refs`。
3. 后端归一化：标题去重、按 `prereqs` 拓扑排序、按最长路径计算 `depth`、去环、id 重编号（`c1..cN` 即学习顺序）、过滤笔记根之外的引用，限制 60 概念 / 5 引用 / 200 字片段。
4. 若识别出 ≥2 个领域且各占 ≥25%（概念总数 ≥4），进入 `AwaitingDomainConfirm`；前端展示领域卡片，可改名或合并后确认。

**轮次（单次合并调用）**：每轮一次 `exploration_turn`，返回 `{verdict, feedback, next_action, hint, next_concept_id, next_question, whiteboard_md, covered_concepts}`。`next_action` 语义：`hint` 只给方向性提示并对同一概念重问（连续提示 3 次后强制推进）；`question` 换角度重问；`advance` 覆盖当前概念并推进；`complete` 结束。JSON 解析失败追加一次 `exploration_repair`，仍失败则不写盘、保留用户输入。

**时间线与撤销**：`groups[]` 保存每轮的问题/回答/判定/白板，`active_group` 是游标；覆盖集合由 `groups[..active_group]` 的 `covered_after` **推导**，不另存权威状态。`/undo` 只移动游标（保留尾部供 `/redo`），重新作答时截断尾部；`/btw` 写入独立的 `asides[]`，不推进游标、不计覆盖；`/skip` 把概念记入 `skipped[]` 并丢弃待答问题；`/end` 标记 `completed_early`。

**白板**：每轮回答后由 LLM 产出一段 Markdown（表格/公式/文本示意），存为 `WhiteboardStep`（含 `group_index`），右栏用现有 MarkdownIt + KaTeX 管线渲染；回放时按游标取对应快照。

**学习史格式**：可见正文（front matter + 缩进知识树 + 逐轮问答 + 旁支 + 未覆盖 + Mermaid 附录）便于人读；文末隐藏 `<!-- knowtequiz:session {...} -->` JSON 附录供无损导入。导入一律创建副本（`origin: imported`），不改写原文件；未知 `format_version` 拒绝。自动导出写入 `<data>/exploration/exports/`；写入笔记库需显式提供裸文件名且拒绝覆盖。

**命令/路由**：见 §6.1 / §6.2。桌面用 Tauri `Channel` 推送初始化事件，Web 用 SSE；前端 `src/services/exploration.ts` 统一 dispatch。

---

## 6. 接口与通信设计

### 6.1 Tauri Command（前端 → 后端）

| Command | 参数 | 返回 | 说明 |
|---|---|---|---|
| `scan_notes` | `root_path: string` | `NoteTreeNode[]` | 扫描目录，返回笔记树 |
| `read_note` | `path: string` | `NoteContent` | 读取单篇笔记内容 |
| `generate_quiz` | `path, types, count, difficulty, lang` | `QuizStream` | 发起出题请求，返回流式 SSE |
| `analyze_mistake` | `question, user_answer, note_path` | `AnalysisResult` | 错题分析 |
| `save_mistake` | `MistakeEntry` | `boolean` | 保存错题 |
| `load_mistakes` | - | `MistakeEntry[]` | 加载全部错题 |
| `submit_answer_advanced` | `question_id, user_answer, user_reasoning, note_path` | `DiagnosisStream` | 高级模式：提交答案+理由，启动诊断流 |
| `diagnose_follow_up` | `session_id, user_reply` | `DiagnosisStream` | 高级模式：提交追问回复，继续诊断 |
| `generate_diagnosis_report` | `session_id` | `DiagnosisReport` | 高级模式：生成最终诊断报告 |
| `get_settings` | - | `Settings` | 读取配置 |
| `save_settings` | `Settings` | `boolean` | 保存配置 |
| `select_folder` | - | `string \| null` | 唤起系统文件夹选择对话框 |
| `convert_document` | `source_path, target_path, format, options` | `ConvertResult` | 纯 Rust 将 PDF/DOCX/PPTX/HTML 转为 Markdown（V1.1） |
| `exploration_create` | `source_paths, title?` | `ExplorationSession` | 创建 `Initializing` 会话 |
| `exploration_start_stream` | `session_id, on_event` | `ExplorationStreamEvent` | 读取 + 摘要 + 图谱；可能先发 `domain_warning` |
| `exploration_confirm_domains` | `session_id, domains, on_event` | `ExplorationStreamEvent` | 确认领域并激活 |
| `exploration_next_question` | `session_id` | `ExplorationSession` | 生成待答问题 |
| `exploration_ask` | `session_id, answer` | `ExplorationSession` | 单次轮次调用 |
| `exploration_undo` / `exploration_redo` | `session_id` | `ExplorationSession` | 游标移动 |
| `exploration_skip` / `exploration_finish` | `session_id` | `ExplorationSession` | 跳过概念 / 提前结束 |
| `exploration_aside` | `session_id, question` | `ExplorationSession` | `/btw` 旁支问答 |
| `exploration_export` / `exploration_save_to_vault` | `session_id[, file_name]` | `ExportResult` / `string` | 自动导出 / 写入笔记库 |
| `exploration_import` | `markdown, source_name?` | `ExplorationSession` | 导入副本（只读） |
| `exploration_continue` | `session_id` | `ExplorationSession` | 从导入/已结束会话继续 |
| `exploration_list` / `get` / `delete` | `[session_id]` | 摘要 / 会话 / `boolean` | 会话管理 |

### 6.2 Tauri Event（后端 → 前端）

| Event | Payload | 触发时机 |
|---|---|---|
| `quiz::chunk` | `QuizQuestion` | LLM 流式返回每道题目时 |
| `quiz::done` | `QuizResult` | 全部题目生成完毕 |
| `quiz::error` | `ErrorMessage` | 出题过程中出错 |
| `diagnosis::chunk` | `DiagnosisMessage` | 诊断分析流式返回 |
| `diagnosis::follow_up` | `FollowUpQuestion` | 需要用户回复新的追问 |
| `diagnosis::report` | `DiagnosisReport` | 最终诊断报告生成完毕 |
| `diagnosis::error` | `ErrorMessage` | 诊断过程中出错 |
| `parser::progress` | `ParserProgress` | 文档解析进度更新（V1.1） |
| `parser::done` | `ParserResult` | 文档解析完成（V1.1） |
| `parser::error` | `ErrorMessage` | 文档解析出错（V1.1） |
| `exploration::phase` | `{ phase }` | 初始化阶段：`reading` / `summarizing:i/n` / `merging` |
| `exploration::domain_warning` | `{ domains, warning }` | 检测到跨领域，阻塞等待确认 |
| `exploration::graph_ready` | `{ graph }` | 图谱生成或领域确认完成 |
| `exploration::done` / `error` | `{ session_id }` / `{ message }` | 初始化结束 |

### 6.3 LLM API 调用规范

**请求格式**（兼容 OpenAI Chat Completion）：

```json
{
  "model": "qwen2.5:7b",
  "messages": [
    { "role": "system", "content": "你是一个知识测验生成助手..." },
    { "role": "user", "content": "基于以下笔记内容，生成5道单选题...\n\n{{note_content}}" }
  ],
  "stream": true,
  "temperature": 0.7
}
```

**返回格式**（要求 LLM 返回可解析的结构化数据，如 JSON）：

```json
{
  "questions": [
    {
      "type": "single",
      "question": "...",
      "options": ["A. ...", "B. ...", "C. ...", "D. ..."],
      "answer": "B",
      "explanation": "..."
    }
  ]
}
```

### 6.3.1 诊断 API 调用规范（高级模式）

**请求格式**（Initial Diagnosis）：

```json
{
  "model": "qwen2.5:7b",
  "messages": [
    { "role": "system", "content": "你是一位严格的学科导师，擅长通过分析学生的推理过程来定位知识盲区。请用中文输出，并严格按 JSON 格式返回。" },
    { "role": "user", "content": "【笔记原文】\n{{note_content}}\n\n【题目】\n{{question}}\n\n【用户答案】{{user_answer}}\n\n【用户推理过程】\n{{user_reasoning}}\n\n请分析用户的推理过程，找出第一个错误环节，判断错误类型（概念理解错误/知识盲区/逻辑跳跃/计算错误/混淆），并提出一个针对性的追问问题。返回 JSON：{\"is_correct\": bool, \"reasoning_review\": [...], \"blind_spot\": {...}, \"follow_up_question\": \"...\"}" }
  ],
  "stream": true,
  "temperature": 0.3,
  "response_format": { "type": "json_object" }
}
```

**关键参数**：
- `temperature: 0.3`：诊断需要稳定、可预测的分析，降低随机性。
- `response_format: { "type": "json_object" }`：强制 LLM 返回 JSON（OpenAI-compatible API 支持）。

---

## 7. 界面布局规范

### 7.1 三栏布局

```
┌──────────────────────────────────────────────────────────────┐
│  顶部标题栏 (拖拽区 + 窗口控制)                                 │
├──────────┬──────────────────────────────────┬────────────────┤
│          │                                  │                │
│  左栏     │           中栏                    │     右栏       │
│  目录树   │         阅读渲染区                 │   功能面板     │
│          │        (Markdown + LaTeX)         │  (出题/错题/设置)│
│  可隐藏   │                                  │    可隐藏      │
│          │                                  │                │
├──────────┴──────────────────────────────────┴────────────────┤
│  底部状态栏 (当前笔记路径、LLM 连接状态、当前字数)                  │
└──────────────────────────────────────────────────────────────┘
```

### 7.2 交互规范

- **左栏**：文件树，支持文件夹折叠/展开，当前阅读文件高亮。
- **中栏**：阅读模式，支持代码高亮、LaTeX 渲染、图片本地路径转绝对路径。
- **右栏**：Tab 切换（出题 / 错题本 / 设置），功能面板内部状态丰富：
  - **出题面板**：题目卡片、模式切换（Basic/Advanced）、选项卡片、理由输入区（Advanced 模式）、诊断对话流（聊天式气泡）、诊断报告卡片。
  - **错题本**：列表展示，支持按模式过滤（只看高级模式错题），点击展开诊断详情。
  - **设置面板**：LLM 配置、答题默认参数（轮次、默认模式等）。
- **栏开关**：提供按钮或快捷键（如 `Ctrl+B` 隐藏/显示左栏，`Ctrl+Shift+B` 右栏），布局状态持久化到 `settings.json`。

---

### 7.3 主题与毛玻璃

**主题预设**：6 套 VS Code 调色板（Dark+ 默认、Dark Modern、Light+、Monokai、Solarized Dark/Light），由 `theme_config.preset` 选择。每套预设写入完整 CSS 变量（背景/侧栏/文字/边框/语义色），不是只换背景色。`preset` 缺失或为 `custom` 时，仅应用用户手动选择的 `background_color` / `accent_color`，以兼容旧设置。

**玻璃材质**（Nocturne 风格）：

- 无色：`rgba(var(--glass-rgb), alpha)`，不随主题给玻璃上色；`--glass-rgb` 深色主题为 `255,255,255`、浅色为 `15,23,42`。
- 着色范围：滑块 0–100 映射到 **0–15%**，默认 40 → 6%；推荐 5–12%。
- 模糊：**默认 40px，范围 0–60**，配合 `saturate(180%)`。
- 方向性边缘光：`inset 0 0 0 1px` 边框 + 顶部高光 + 底部暗缘 + 外层深阴影；`--glass-border` / `--glass-shadow` 按主题 kind 注入。
- 颗粒：玻璃开启时全屏叠加 2.5% SVG `feTurbulence` 噪点（`body[data-glassmorphism]::after`）。

**层级（关键）**：背景场景必须是 `fixed inset-0 z-0`，应用内容包在 `relative z-10` 中。场景 = 壁纸/视频；没有壁纸时由主题 accent 生成 2–3 个大半径柔光斑。不要把场景放在负 z-index 且让根节点保留不透明背景，否则面板永远透不出场景。

**可读性**：保留 `theme_config.overlay_opacity`（0–50%）作为壁纸压暗层；花哨壁纸建议 10–20%。**性能**：面板使用 `contain: content`，不要对 `backdrop-filter` 做动画。

---

## 8. 开发路线图

### V1.0 — 严格 MVP（当前目标 — 已基本完成）

- [x] 项目脚手架：Tauri + Vue 3 + Vite + Pinia
- [x] 基础布局：三栏结构 + 栏显隐控制
- [x] 笔记浏览：文件夹选择、目录树、文件读取
- [x] Markdown 阅读渲染：MarkdownIt + KaTeX + 代码高亮
- [x] LLM 配置面板：API 地址、模型、Key 输入与测试（含结构化连接测试结果）
- [x] 实时出题：选择题/简答题生成，支持多选答案
- [x] 基本模式答题交互：题目展示、答案选择/输入、即时判分反馈
- [x] **高级模式答题交互（核心功能）**：理由输入、AI 多轮诊断对话、诊断报告生成
- [x] 答题模式切换：Basic / Advanced 模式选择和配置
- [x] 诊断对话 UI：聊天式对话流、追问回复、诊断报告卡片
- [x] 错题本：记录错题（含低级/高级模式完整上下文）、列表展示、分页、按模式/笔记过滤
- [x] 设置持久化：`settings.json` 读写（含 version 字段）
- [x] 打包与发布：Windows / macOS / Linux 安装包

**V1.0 增强（超出原始 MVP 范围，已实现）：**

- [x] 原子 JSON 写入（tmp → sync → rename → .bak 自动恢复）
- [x] 诊断 session 持久化（落盘到 `sessions/`，支持跨启动恢复）
- [x] 数据备份/恢复（手动备份 + 恢复最新备份）
- [x] 数据状态仪表盘（文件大小、修改时间、存在性检查）
- [x] 阅读位置恢复（为每篇笔记记住滚动位置）
- [x] 目录扫描忽略规则（跳过 `.git`、`node_modules` 等）
- [x] 工作区状态持久化（rootPath、展开目录、选中文件）
- [x] JSON payload 校验（拒绝无效题目、空响应、格式不匹配）
- [x] Web server 安全性（默认绑定 `127.0.0.1`、限制 CORS）
- [x] LLM 连接测试统一走服务层（移除 store 直连）
- [x] 错题本搜索、过滤、导出、复习流程、盲区标签过滤
- [x] 笔记内搜索（Ctrl+F）与大纲目录（TOC）
- [x] 解析失败自动修复（单次 LLM 重试）
- [x] LLM 能力探测（模型列表、stream 支持）
- [x] 错题 JSONL 迁移（追加写入，自动从 legacy json 迁移）
- [x] 增量笔记索引（size+time 判断，跳过未变化文件）
- [x] 本地质量门禁（Husky + Prettier + lint-staged）
- [x] CI/CD（GitHub Actions：type-check + test + build + Rust test + clippy + 冒烟）
- [x] 冒烟测试脚本（PowerShell，9 步覆盖核心路径）
- [x] 发布检查清单与备份恢复文档
- [x] 单元测试（当前 189 Rust + 269 前端测试用例，42 个前端测试文件）

### V1.1 — 文档智能增强

- [x] **PDF / Word / PPT / HTML → Markdown 转换（纯 Rust，无外部运行时）**（2026-10-12 完成）
  - [x] `doc_parser` 分派与 `ParsedDocument` 统一输出契约（Markdown + 图片 + 标题 + warnings）
  - [x] `ooxml` 子模块：DOCX/PPTX 解析（`zip` + `quick-xml`，提取 `w:t`/`a:t`、`w:pStyle` 标题、`w:tbl` 管道表格、`a:blip` 图片引用）
  - [x] `pdf` 子模块：PDF 文本层抽取（`pdf-extract`）；无文本层时提示"扫描件，OCR 未启用"
  - [x] `html` 子模块：无依赖的 HTML→MD（`#` 标题、段落、`-` 列表，跳过 script/style/head）
  - [x] 前端：`DocumentEmptyState` 提供转换按钮 / 打开笔记 / 重新转换（含覆盖提示），并展示 warnings
  - [x] 文件树让非 MD 文件可见（`fs_service` 按白名单放行 PDF/DOCX/PPTX/HTML，淡化显示 + 类型角标）
- [ ] **OCR 扫描件识别（Rust + ONNX + RapidOCR）** — 与 `doc_parser` 解耦：PDF 无文本层时触发 OCR 分支；模型按需下载（**升级项，v1 未做**）
- [ ] 用户自定义提示词模板引擎（JSON 模板，变量替换）
- [ ] 错题复习模式（随机抽错题重练）
- [ ] 笔记内搜索（全文检索，基于 `grep` 或简易索引）

### V1.2 — 完整版（远期规划）

- [x] **探索性学习模式（2026-10-23 完成）**：模式入口与三栏复用、map-reduce 知识图谱、跨领域阻塞确认、单次轮次问答、白板快照、`/undo` `/redo` `/btw` `/skip` `/end`、学习史导出/导入（隐藏 JSON 附录 + 可见正文）、步进回放、导入副本与「从此继续」
- [ ] 批量出题（多文件/多目录批量生成题库）
- [ ] 高级模板系统（教育理论插件、弱项分析深度报告）
- [ ] 笔记标签系统
- [ ] 数据导出/备份（JSON 压缩包）
- [ ] 插件系统（Rust dylib / WASM 扩展）

---

## 9. 风险与应对

| 风险 | 影响 | 应对策略 |
|---|---|---|
| **本地 LLM 推理慢/不可用** | 出题功能不可用 | 支持任意兼容 OpenAI 的远程 API；提供 API 连接测试与错误提示 |
| **Markdown 渲染不一致** | 阅读体验差 | 采用成熟库（MarkdownIt），对边缘语法宽容处理，不追求 100% 标准兼容 |
| **大目录扫描慢** | 启动延迟 | V1.0 容忍；V1.1 引入轻量索引或异步后台扫描 |
| **OCR 模型体积大** | 安装包膨胀 | RapidOCR 模型按需下载（首次使用时引导下载），不打包进安装包 |
| **Python 环境缺失** | 文档转换功能不可用 | ~~Python 方案风险~~ — 已改为纯 Rust，无外部运行时依赖，此风险移除 |
| **PDF 复杂排版解析错乱** | 转换质量差 | 纯 Rust 文本层抽取保内容不保版式；表格/多栏/图片另列为升级项；明确告知用户"转换结果可能需要手动校对"，提供原始文件并行查看 |
| **扫描件 PDF 无文本层** | 转换结果为空 | 检测无文本层 → 走 `ocr_engine` ONNX 分支（模型按需下载），未装模型时明确提示而非静默返回空 |
| **Tauri v2 生态成熟度** | 插件/文档不完善 | 优先使用 Tauri 官方插件，社区插件谨慎评估，必要时自行实现 |

---

## 10. 目录结构（预期）

```
KnowteQuiz/
├── src/                          # 前端 Vue 3 源码
│   ├── components/               # 通用组件
│   │   ├── Layout/               # 布局相关（三栏、标题栏、状态栏）
│   │   ├── Explorer/             # 目录树组件
│   │   ├── Reader/               # 阅读渲染组件
│   │   ├── Panel/                # 右侧面板容器
│   │   ├── Quiz/                 # 出题/答题/结果组件
│   │   └── common/               # 按钮、输入框等基础组件
│   ├── views/                    # 页面级组件（如有路由）
│   ├── stores/                   # Pinia Stores
│   │   ├── explorer.ts           # 笔记目录状态
│   │   ├── reader.ts             # 当前阅读状态
│   │   ├── quiz.ts               # 出题/答题状态
│   │   └── settings.ts           # 全局配置状态
│   ├── services/                 # 前端服务层（调用 Tauri Command）
│   │   ├── tauri.ts              # Tauri invoke 封装
│   │   ├── note.ts               # 笔记相关 API
│   │   ├── quiz.ts               # 出题相关 API
│   │   └── settings.ts           # 配置相关 API
│   ├── types/                    # TypeScript 类型定义
│   ├── styles/                   # 全局样式、主题变量
│   ├── utils/                    # 工具函数
│   ├── App.vue
│   └── main.ts
├── src-tauri/                    # Tauri Rust 后端
│   ├── src/
│   │   ├── main.rs               # 入口
│   │   ├── lib.rs                # 库入口（可被测试）
│   │   ├── commands/             # Tauri Commands（前端可调用的函数）
│   │   │   ├── note.rs
│   │   │   ├── quiz.rs
│   │   │   └── settings.rs
│   │   ├── services/             # 业务逻辑层
│   │   │   ├── fs_service.rs     # 文件系统扫描与读取
│   │   │   ├── note_service.rs   # 笔记元数据提取
│   │   │   ├── quiz_engine.rs    # 提示词拼接与 LLM 调用
│   │   │   ├── storage.rs        # JSON 存储读写
│   │   │   ├── config.rs         # 配置管理
│   │   │   ├── doc_parser.rs     # 文档解析分派（OOXML/PDF/HTML → Markdown，V1.1）
│   │   │   │   ├── ooxml.rs      # DOCX/PPTX 解析（zip + quick-xml）
│   │   │   │   └── pdf.rs        # PDF 文本层抽取（pdf-extract/lopdf）
│   │   │   └── ocr_engine.rs     # OCR 引擎（ONNX，扫描件升级路径，V1.1）
│   │   ├── models/               # 数据结构定义（Rust struct）
│   │   └── utils/                # 工具函数
│   ├── Cargo.toml
│   └── tauri.conf.json
├── public/                       # 静态资源
├── docs/                         # 项目文档
│   ├── TECH_SPEC.md              # 本文档
│   └── API.md                    # 接口文档（可选）
├── tests/                        # 测试文件（前端 + Rust 单元测试）
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tailwind.config.js            # 如使用 Tailwind
└── README.md
```

---

## 附录 A：Grill-Me 决策摘要

以下决策均通过 `/grill` 流程与项目 owner 逐条确认：

1. **桌面框架**：Tauri（放弃 Electron，追求轻量化）
2. **文档解析（2026-10-12 更新，取代原 Python 方案）**：**纯 Rust**（无外部运行时）—— OOXML（DOCX/PPTX）用 `zip` + `quick-xml`，PDF 用 `pdf-extract`/`lopdf` 抽文本层。弃用原"Python 子进程 + pdfplumber/python-docx"方案。理由：(a) 与"纯本地、极致轻量"定位一致，不给用户装 Python 的负担；(b) 原方案未覆盖 PPT，OOXML 天然覆盖 DOCX+PPTX；(c) 表格/版式高保真与扫描件 OCR 单列为后续升级项，避免第一版就背上外部依赖。**OCR 保持 Rust ONNX 方案**（`ort`），且与 `doc_parser` 解耦以保留升级空间。
   - **入口与可见性（2026-10-12 确认，方案 A）**：非 MD 文档在**文件树中可见**（淡化显示，仅白名单扩展名 PDF/DOCX/PPTX/HTML，其余仍隐藏），点击不渲染为笔记而是打开「转换为 Markdown」空态页；转换后源文件保留并打「已转换」徽标以记录血缘。拒绝方案 B（文件选择器导入，功能不可发现、产物游离于笔记库外）与方案 C（右键/拖拽，Web 运行时无原生菜单需自绘）。前置改造：`fs_service.rs:190` 当前只推 `.md`/`.markdown`，需放行白名单文档。
   - **产物命名（2026-10-12 确认，方案 D）**：产物为**与源同级、保留源扩展名**的 `design.docx.md`。纯函数映射 `X.ext → X.ext.md`，故「已转换」徽标与重新转换均无需任何映射表——前者判定目标是否存在，后者幂等覆盖同一目标。树中展示名自动为 `design.docx`（`FileTreeItem` 仅剥末尾 `.md`），**显式标出出处**，且永不覆盖用户手写的 `design.md`，也不与 `design.pdf.md` 冲突。拒绝 A（覆盖用户笔记 + 一对多不可反推）、B（`.converted` 后缀仍跨格式冲突）、C（子目录使血缘依赖目录约定、引入新扫描根）。
   - **v1 内容保真边界（2026-10-12 确认，方案 B；同日修正图片落盘）**：**正文 + 标题层级 + 图片 + DOCX 管道表格（合并单元格降级为纯文本）**。**图片落盘于与产物同级的派生目录**（`design.docx.md` → `design.docx.images/image001.png`），MD 中用**相对路径**引用——派生目录在笔记 root **之内**，故复用已有通路（Web 资产路由 + 桌面 `convertFileSrc`）即可工作；随笔记库移动/备份/迁移均成立；与产物同用「源扩展名前缀」的派生命名，血缘纯函数可推。**PDF 表格与复杂版式明确进升级项**（`pdf-extract` 无表格能力），对外承诺："标题、正文、图片、DOCX 简单表格；PDF 复杂版式可能丢失，原文件可并行查看"。
     - **拒绝原记录的 `app-data/media/` 方案**（经 `markdownAssets.ts` 交叉验证）：(a) `read_note_asset_handler` 已绑定笔记 root，media 在 root 外 → Web 模式资产路由 400、图片全裂；(b) 绝对路径写进 MD → 换机器即失效；(c) 备份域错位——笔记库是用户备份域、app-data 是应用备份域。**同时拒绝 data URL 内联**：笔记正文会进 LLM prompt，base64 图会撑爆上下文并污染 `index.json`/git diff。
     - **拒绝 A**（PPT 只剩 bullet，等于没支持）与 **拒绝表格全保真**（合并/嵌套单元格需降级，PDF 表格本就无 Rust 方案）。
   - **备份策略（2026-10-12 确认，方案 D：维持现状不备份 `media/`）**：配图已随笔记库走，受用户自己的笔记库备份覆盖（笔记库=用户备份域）；app-data `media/` 此后仅存主题图片/视频等**可重选偏好**，非数据，故不进 `MANAGED_DATA_FILES`。**由此撤销**前一轮基于「配图在 media/」推导出的「备份图片、排除视频」（原方案 C）——其前提已不存在。
   - **过期提示（2026-10-12 确认，方案 B）**：产物视为**可被用户编辑的笔记**，因此**不做自动重转**。改为基于 mtime 比对（产物早于源）把徽标升级为「可能过期」并提供**一键重转**——若产物已被编辑则先提示"将覆盖你的修改"。复用 `index.json` 增量扫描已有的 size+mtime 基建，成本极低。拒绝 A（用户长期读到过期内容却无从知晓）、C（自动覆盖会毁掉用户对转换结果的校对与补图 = 数据丢失）、D（哈希冲突判定复杂且仍需"产物是否被编辑"的基线）。
3. **LLM 模式**：API 连接。兼容 OpenAI 格式，用户自行配置本地（Ollama）或远程端点。
4. **OCR 引擎**：Rust + ONNX Runtime (`ort`)，默认 RapidOCR 模型。支持可配置其他引擎路径，不做 CLI 兼容。
5. **前端框架**：Vue 3 + TypeScript（放弃 React/Svelte）
6. **Markdown 渲染**：Obsidian 风格阅读模式，对标沉浸式阅读体验。
7. **Markdown 编辑**：参考成熟开源方案（Vditor/Milkdown），作为辅助功能。
8. **数据存储**：纯 JSON 文件（放弃 SQLite），便于阅读和迁移。
9. **笔记索引**：启动时仍会扫描目录树，已添加忽略规则（`.git`、`node_modules` 等），已实现 workspace 状态持久化（rootPath、展开目录、选中文件、阅读位置）。扫描会写入轻量 `index.json`，记录笔记路径、标题、大小和修改时间，坏编码笔记使用文件名标题兜底，索引文件纳入数据状态和备份。
10. **出题模式**：实时出题为主（无持久化题库），支持丰富参数（难度/语言/题型/知识框架）。
11. **提示词引擎**：Rust 后端处理模板拼接（V1.1 实现完整模板引擎，V1.0 仅内置固定模板）。
12. **布局**：三栏式（VS Code / Obsidian 风格），支持灵活开关各栏。
13. **V1.0 范围**：严格 MVP，聚焦"阅读 + 基础实时出题 + 错题记录"。
14. **答题模式**：双模式架构（低级 Basic / 高级 Advanced），高级模式为核心差异化功能。
15. **高级模式机制**：用户必须写出推理理由 → AI 多轮诊断对话 → 生成知识盲区报告。
16. **诊断轮次**：默认 3 轮，可配置，防止 LLM 调用成本过高。
17. **错题存储**：高级模式错题保存完整诊断对话和报告，支持回溯知识盲区。
18. **探索性学习（2026-10-23，14 项决策）**：入口在 `Explorer` 底部；初始化一律 map-reduce；白板纯 Markdown + KaTeX（零依赖 `mermaidBridge` 仅做文本互转）；每轮 1 次合并调用 + 1 次修复；`/undo` 单轮回退 + `/redo`；`/btw` 入导出、回放可跳过；仅 `.md` 可作源；导出以缩进树为权威并附 Mermaid；全覆盖自动完成 + 允许提前结束；自动导出到 app-data，写笔记库需显式文件名；导入创建副本；单活动会话可恢复；跨领域阻塞确认。**实施偏差**：导入以文末隐藏 JSON 附录做无损还原（正文仍人类可读）；摘要直接存会话而非独立缓存；`/hint` 未单独提供（「不知道」路径已覆盖）；另存笔记库使用裸文件名（无原生对话框）。

---

*文档版本：v1.2.0*
*最后更新：2026-10-23*
*决策状态：V1.0 + V1.1 + 探索性学习全部完成，详见 `docs/optimization-plan.md` 与 `docs/exploration-mode-plan.md`*
