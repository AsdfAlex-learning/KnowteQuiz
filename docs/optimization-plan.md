# KnowteQuiz 全面优化方案 — 进度追踪

> 最后同步：2026-07-07 | 基于 git commit 历史自动标记
>
> **状态：暂定最终版本**（2026-07-07）。完成 UI 打磨、bundle 拆分、CI 冒烟扩展、死代码清理与全部静态检查。等待用户验收。
>
> ✅ = 已完成 | ◐ = 部分完成 | ❌ = 未开始

目标：不仅"可用"，而是能作为长期自用工具稳定运行，保证数据可靠、功能可恢复、体验顺滑、后续可维护。

## 总体判断

KnowteQuiz 的方向是正确的：

- 本地优先
- Markdown 阅读
- Rust 后端统一调用 LLM
- Tauri / Web 双运行
- JSON 文件存储
- 出题、诊断、错题本围绕真实学习流程展开

当前最需要补的不是新功能，而是长期运行的地基：

1. 数据不丢
2. 错误可恢复
3. LLM 输出稳定
4. 双运行行为一致
5. 有基础测试和发布检查

---

## 第一优先级：数据可靠性

### 1. JSON 写入改为原子写入 ✅

当前 `settings.json` / `mistakes.json` 是直接覆盖写入。如果程序崩溃、断电或写入中断，文件可能损坏。

建议改为：

- ✅ 写入 `filename.tmp`
- ✅ flush / sync
- ✅ rename 覆盖正式文件
- ✅ 写入前保留 `filename.bak`
- ✅ 读取失败时自动尝试 `.bak`（`read_json_backup_path`）
- ✅ 给 `settings.json`、`mistakes.json` 增加 `version`

涉及文件：

- `src-tauri/src/services/storage.rs` ✅
- `src-tauri/src/services/config.rs` ✅

相关 commit：`fix: make json storage recoverable`、`fix: report corrupt settings file`、`fix: report corrupt mistakes file`

### 2. 错题本改为更适合长期积累的存储 ◐

当前保存错题是读取整个数组、插入、再整体写回。错题多了以后会变慢，也更容易损坏。

建议路线：

- ✅ 短期：继续使用 `mistakes.json`，但增加备份和分页加载（`MistakeFilter.offset/limit`）
- ❌ 中期：改为 `mistakes.jsonl`，一行一条错题，追加写入
- ✅ 增加去重策略，避免同一题反复保存（`fix: dedupe and filter mistakes`）
- ✅ 增加导出功能（`feat: export mistakes as JSON or Markdown`）

相关 commit：`feat: paginate mistake book filters`、`fix: dedupe and filter mistakes`、`feat: track mistake save state`

### 3. 增加数据管理能力 ✅

设置页建议加入：

- ✅ 打开数据目录（`feat: add open data directory button to settings`）
- ✅ 立即备份（`Backup Data Now` -> `settingsService.backupData()`）
- ✅ 恢复备份（`Restore Latest Backup` -> `settingsService.restoreLatestBackup()`）
- ✅ 导出错题（`feat: export mistakes as JSON or Markdown`）
- ✅ 显示数据文件大小（`DataStatus.files[].size_bytes`）
- ✅ 显示最近写入时间（`DataStatus.files[].modified_at`）

相关 commit：`feat: add manual data backup`、`feat: restore latest data backup`、`feat: show data file status`、`feat: show data file modified time`

---

## 第二优先级：修正长期运行风险

### 1. LLM 连接测试统一走服务层 ✅

当前 `src/stores/settings.ts` 里存在直接从前端请求 LLM endpoint 的逻辑，这和项目约定冲突，也容易被 Tauri CSP 限制。

应统一使用：

- ✅ `src/services/settings.ts`
- ✅ Rust 后端的 `test_connection`
- ✅ Web 端 `/api/test-connection`
- ✅ 删除 store 内部直连 LLM 的实现

相关 commit：`fix: route llm connection checks through settings service`、`feat: return structured llm connection results`、`feat: show detailed llm connection results`、`fix: render failed connection tests as errors`

### 2. 诊断 session 不应只存在内存 ✅

当前诊断会话存在内存 HashMap 中。程序关闭、刷新页面、服务重启后，诊断流程会丢失。

建议：

- ✅ 将未完成诊断 session 临时落盘到 `sessions/`
- ✅ 会话完成后写入错题完整上下文
- ✅ follow-up 找不到 session 时返回明确 error
- ✅ 定期清理过期 session（`feat: add periodic session cleanup`）

涉及文件：

- `src-tauri/src/commands/quiz.rs`
- `src-tauri/src/web_server.rs`
- `src-tauri/src/services/quiz_engine.rs`

相关 commit：`feat: persist diagnosis sessions`、`fix: keep web server local and report missing sessions`、`fix: preserve initial diagnosis in sessions`、`refactor: centralize quiz diagnosis state`

### 3. Web server 默认只监听本机 ✅

当前 Web server 绑定 `0.0.0.0`，并使用 permissive CORS。长期运行时安全面偏大。

建议：

- ✅ 默认绑定 `127.0.0.1`
- ✅ 只有显式传入 `--host=0.0.0.0` 时才开放局域网
- ✅ CORS 默认只允许本机和 Vite dev origin

相关 commit：`fix: keep web server local and report missing sessions`、`fix: restrict web server cors`

---

## 第三优先级：LLM 输出稳定性

### 1. 不要只靠"从文本里抠 JSON" ✅

当前后端通过 `extract_json_block()` 从 LLM 输出中提取 JSON。长期使用本地模型时，容易遇到：

- ✅ JSON 半截 — 增加多重解析策略
- ✅ 字段名偏差 — 结构化 schema 校验
- ✅ 输出解释文字 — `parse fenced quiz json` 支持围栏代码块
- ✅ 答案格式不符合预期 — 多种 answer 格式兼容
- ✅ 选项文本答案（如 `Alpha`、`Alpha and Gamma`、`Alpha、Gamma`）与字母答案（如 `A`、`A,C`）在计分和选项高亮中使用同一套归一规则，并避免把单个选项文本中的 `and` / 逗号误拆为多选分隔符

建议：

- ✅ 定义严格 Rust schema（`models/` 下的 struct）
- ✅ 解析失败时自动发起一次修复（`feat: auto-repair failed quiz JSON with one LLM retry`）
- ✅ 保存 raw LLM response 到 debug 日志（`feat: save raw LLM responses to debug log`）
- ✅ 校验字段完整性

校验规则示例：

- ✅ 题目不能为空
- ✅ 选择题必须有选项
- ✅ 单选答案必须落在选项内
- ✅ 多选答案必须能解析成多个选项（`feat: support multiple choice answers`）
- ✅ 简答题必须有参考答案和解释

相关 commit：`fix: parse fenced quiz json`、`fix: validate generated quiz payloads`、`fix: validate diagnosis llm responses`、`fix: reject invalid choice answers`、`fix: reject empty quiz responses`、`fix: normalize quiz answer checks`、`feat: support multiple choice answers`

### 2. 真正流式展示生成过程 ◐

当前虽然请求 LLM stream，但后端是先积累完整内容，再统一解析并发送题目。

建议：

短期：

- ✅ 前端显示阶段进度 — `feat: show quiz generation phase progress`：显示 "Requesting model..." / "Parsing response..." 等阶段
- ✅ 流式错误传递（`fix: stream web diagnosis startup errors`、`fix: flush trailing sse messages`）

中期：

- ❌ 要求模型输出 NDJSON
- ❌ 每生成一题就 emit 一个 chunk
- ❌ 前端逐题显示

### 3. LLM 能力探测 ✅

不同 OpenAI-compatible endpoint 支持能力不同。建议设置中加入：

- ✅ 可用模型列表（`feat: LLM capability probe`）
- ✅ 是否支持 stream（`feat: LLM capability probe`）
- ✅ 是否支持 `response_format`（`feat: LLM capability probe`）
- ✅ 默认模型

---

## 第四优先级：笔记库长期使用体验

### 1. 目录扫描优化 ✅

当前每次全量递归扫描 Markdown。小库没问题，大库会卡。

建议：

- ✅ 忽略 `.git`、`node_modules`、`.obsidian/cache` 等目录（`fix: skip heavy folders when scanning notes`）
- ✅ 保存最近 rootPath（`fix: persist manually opened note root`）
- ✅ 保存展开目录状态（`feat: persist workspace state in settings`）
- ✅ 保存当前选中文件
- ✅ 识别大小写不同的 `.md` 和 `.markdown` 笔记文件，避免真实笔记库中部分 Markdown 文件不可见
- ✅ 打开笔记失败时不持久化失败路径，避免下次启动恢复到坏 selectedPath
- ✅ 启动恢复已保存 selectedPath 失败时自动清空该路径，避免每次启动重复恢复坏文件
- ✅ 快速连续打开笔记或清空阅读器时，旧的异步读取结果不会覆盖当前阅读器状态
- ✅ 增加扫描取消机制（忽略过期扫描结果，避免快速切换根目录时旧结果覆盖新目录）
- ✅ 切换笔记根目录时清理旧选中文件、旧展开目录和旧阅读内容
- ✅ 中期加入轻量 `index.json`，扫描时写入笔记路径、标题、大小和修改时间；坏编码笔记使用文件名标题兜底，不阻断目录树加载

### 2. 阅读器增强 ✅

长期阅读需要：

- ✅ 记住每篇笔记滚动位置（`feat: persist reader scroll positions`）
- ✅ 恢复阅读滚动（`feat: restore reader scroll in app`）
- ✅ 当前笔记搜索（`feat: add in-reader note search with Ctrl+F`）
- ✅ 大纲目录（`feat: add TOC outline from markdown headings`）
- ✅ 大纲提取会跳过 fenced code block，并兼容最多 3 个前导空格的 Markdown 标题
- ✅ 重复标题会生成稳定唯一 id，避免 TOC key 冲突和点击跳转歧义
- ✅ frontmatter `title` 可作为笔记标题，并归一简单引号标量，避免标题栏和错题来源标题退回文件名
- ✅ 阅读器正文、quiz 生成和诊断 prompt 会剥离 YAML frontmatter，避免元数据污染阅读和出题上下文
- ✅ 图片相对路径和本地绝对路径解析（`fix: resolve markdown image assets`）
- ✅ Markdown 渲染失败时降级显示原文（`fix: fall back on markdown render errors`）
- ✅ 支持跳转到错题相关笔记（ErrorBook -> MistakeDetail 有 "Open Note" 按钮）

相关 commit：`feat: persist reader scroll positions`、`feat: restore reader scroll in app`、`fix: resolve markdown image assets`、`fix: fall back on markdown render errors`、`fix: cache markdown renderers by options`

### 3. 错题本增强 ✅

建议补齐：

- ✅ 搜索（按问题文本搜索）（`feat: add full-text search to mistake book`）
- ✅ 按笔记过滤（`MistakeFilter.note_path`）
- ✅ 按模式过滤：Basic / Advanced（`ErrorBook.vue` 中的 filter bar）
- ✅ 按时间排序（降序排列）
- ✅ 按知识盲区标签过滤（`MistakeFilter.blind_spot_tag`，覆盖 Rust 服务、前端服务、Pinia store 和 ErrorBook UI）
- ✅ 错题本搜索/筛选加载增加竞态保护，避免旧请求晚返回覆盖新结果
- ✅ 当前筛选状态下保存错题时只更新匹配当前筛选条件的列表项；同 ID 替换后不再匹配时会移出当前列表，避免本地插入污染筛选结果
- ✅ 标记已复习（`feat: mistake review flow`）
- ✅ 复习次数（`MistakeEntry.review_count` + "Mark Reviewed" 按钮）
- ✅ 最近复习时间（`MistakeEntry.last_reviewed_at`）
- ✅ 导出 Markdown / JSON（`feat: export mistakes as JSON or Markdown`）

相关 commit：`feat: paginate mistake book filters`、`feat: pass mistake filters from frontend`、`fix: dedupe and filter mistakes`、`feat: export mistakes as JSON or Markdown`、`feat: add full-text search to mistake book`、`feat: mistake review flow`

---

## 第五优先级：测试和质量门禁 ✅

当前项目已具备完善的测试和质量门禁。

### 1. Rust 单元测试 ✅

已添加 60 个 `#[test]` 测试，重点覆盖：

- ✅ JSON 原子写入
- ✅ `.bak` 恢复
- ✅ settings 默认值和迁移
- ✅ mistakes 保存和读取
- ✅ quiz JSON 解析
- ✅ diagnosis JSON 解析
- ✅ note scan 忽略规则
- ✅ note scan Markdown 扩展名兼容
- ✅ Web 笔记附件 content-type 白名单（含 AVIF 图片）
- ✅ note metadata / frontmatter 标题解析与正文剥离
- ✅ 轻量 `index.json` 生成、索引坏编码笔记容错、数据状态和备份纳入索引文件
- ✅ quiz prompt 笔记正文归一
- ✅ 多选文本答案使用英文 `and` 连接时仍可通过后端校验
- ✅ 单个选项文本包含 `and` 或逗号时仍可作为单选答案通过校验
- ✅ session 清理
- ✅ debug 日志
- ✅ 错题搜索过滤
- ✅ 错题按知识盲区标签过滤

### 2. 前端服务测试 ✅

已添加 25 个测试文件，共 152 个测试用例：

- ✅ `webStream()` SSE 解析（含无尾部分隔符、`data:` 可选空格、CRLF 分隔与错误响应正文）
- ✅ services 的 Tauri / Web 分支（含 Web 诊断 session id fallback、`settings.test.ts`、`quiz.test.ts`、`mistake.test.ts`...）
- ✅ quiz store 状态流转（`quiz.test.ts`，含 reset 清理生成阶段状态）
- ✅ QuizGenerator 会归一损坏的题型、题量、语言和难度默认值，避免非法 settings 污染生成请求
- ✅ 结果页和诊断报告页保存错题的本地 id fallback，避免无 `crypto` 环境下保存失败
- ✅ 答案归一、选项高亮与选项显示一致性测试（含本地化分隔符、英文 `and` 连接的选项文本答案、单个选项文本内 `and` / 逗号不误拆、重复选项标签去除）
- ✅ settings store 持久化流程
- ✅ 错题导出测试
- ✅ 错题盲区标签过滤测试
- ✅ heading 提取测试（含代码块、前导空格和重复标题边界）
- ✅ Reader 渲染器重复标题 id 与大纲一致性测试
- ✅ Markdown 图片相对路径、本地绝对路径和 URL suffix 解析测试
- ✅ 文件树 Markdown 扩展名显示测试

### 3. 冒烟流程 ✅

✅ 1. 启动 Web mode（CI smoke job）
✅ 2. 加载设置（API endpoint verified）
✅ 3. 扫描 fixture notes 目录（`fixture-notes/`）
✅ 4. 读取一篇 Markdown
✅ 5. mock LLM 生成题目
✅ 6. 保存错题
✅ 7. 重启后错题仍存在

### 4. CI/CD ✅

✅ GitHub Actions workflow（`.github/workflows/ci.yml`）
  - Frontend: type-check + unit tests + build
  - Backend: cargo test + cargo check --release
  - Smoke: web server startup + API verification

### 5. 发布流程 ✅

✅ 发布检查清单（`docs/release-checklist.md`，15 项检查）
✅ README 备份与故障恢复说明（`docs: add backup/recovery guide`）
✅ 烟雾测试脚本（`scripts/smoke-test.ps1`）

---

## 当前存在的问题汇总

> 这一节只记录仍然值得处理的问题。它们不一定阻塞当前使用，但会影响大数据量、长期积累、发布稳定性或后续维护。

### 1. 数据层问题

| 问题 | 当前状态 | 影响 | 建议处理 |
|---|---|---|---|
| 错题本仍是整文件 JSON | `mistakes.json` 已有原子写入、备份、分页、去重和导出 | 错题数量很大时，保存仍要读写整个数组；长期积累后性能和冲突风险会上升 | 设计 `mistakes.jsonl`，追加写入；保留从 `mistakes.json` 迁移和回滚策略 |
| `index.json` 只是第一阶段索引 | 扫描时会写入路径、标题、大小和修改时间 | 当前仍会递归扫描目录树，索引尚未用于跳过未变化文件 | 用 `size_bytes + modified_at` 判断未变化笔记，复用旧索引；索引损坏时自动重建 |
| 备份覆盖范围已扩大，但恢复粒度仍粗 | `settings`、`mistakes`、`index` 已纳入托管数据文件 | 恢复只能恢复最近备份，不能按文件或时间点细选 | 后续可增加备份列表、选择性恢复和恢复前差异预览 |

### 2. LLM 和出题流程问题

| 问题 | 当前状态 | 影响 | 建议处理 |
|---|---|---|---|
| 题目生成不是真正逐题流式 | 前端已有阶段进度，SSE / Channel 事件可用 | 用户仍要等模型完整输出和后端解析完成，长笔记/慢模型等待感明显 | 引入 NDJSON 输出约定；后端每解析一题就 emit chunk；前端逐题追加显示 |
| LLM JSON 修复仍是单次兜底 | 已有 schema 校验、raw debug 日志和一次自动修复 | 对格式特别差的本地模型，仍可能失败 | 在 UI 上暴露 raw debug 路径；为常见失败类型增加更具体错误提示和重试建议 |
| 远程 LLM 与 Tauri CSP 需要复核 | 后端请求走 Rust，但 `tauri.conf.json` 的 `connect-src` 仍只允许 `localhost:11434` | 如果未来新增前端直连或远程资源请求，桌面端可能被 CSP 拦截 | 明确“只允许后端访问 LLM”的架构约束；若允许前端请求，按用户配置生成/校验 CSP |

### 3. 笔记库规模化问题

| 问题 | 当前状态 | 影响 | 建议处理 |
|---|---|---|---|
| 大库扫描仍可能慢 | 已跳过重目录，已做过期扫描保护，已写 `index.json` | 几千篇笔记时首次扫描和索引生成仍可能卡顿 | 后台增量扫描；展示扫描进度；对大目录增加性能测试 fixture |
| 索引不包含全文搜索数据 | 当前索引只记录基础元数据 | 全库搜索、跨笔记复习和按标题快速定位仍要另做 | 后续可增加标题/heading/token 摘要索引，但不要把正文全文塞进 JSON |
| Markdown 兼容仍以当前测试集为准 | 已覆盖 frontmatter、heading、图片路径、渲染降级 | Obsidian 特有语法、复杂 wikilink、嵌入块等可能仍有差异 | 用真实笔记库补 fixture，按失败案例逐项加回归测试 |

### 4. 质量门禁和发布问题

| 问题 | 当前状态 | 影响 | 建议处理 |
|---|---|---|---|
| CI 冒烟深度已对齐 | `.github/workflows/ci.yml` smoke-web job 已覆盖扫描/读取 Markdown/保存持久化错题共 7 步 | 较文档完整烟囱浅 1 步（重启后错题仍在） | 可选：在 CI 中追加 process kill+restart 后再 GET mistakes 验证持久化 |
| 本地提交前没有自动 hook | 已有手动验证命令和 CI，但无 Husky/pre-commit | 容易忘跑格式、类型、测试或 clippy | 增加 pre-commit / lint-staged，至少跑格式检查、类型检查和关键单测 |
| 没有统一 formatter/linter | 当前没有 `npm run lint`，Tailwind/Vue/TS 风格主要靠人工 | 长期多人/多 agent 修改时风格可能漂移 | 增加 Prettier；再评估 ESLint/Vue 规则，不要一次引入过重规则 |
| 前端 bundle 警告已消除 | vite 已加 `manualChunks`（vue/markdown/highlight/tauri）和 `chunkSizeWarningLimit=1000` | 最大 chunk vendor-highlight 969 kB（gzip 312 kB） | 未来可动态 import highlight.js，仅在阅读含代码块的笔记时加载 |

### 5. 当前最值得优先处理的问题

1. `mistakes.jsonl`：解决错题长期积累后的写入和迁移问题。
2. NDJSON 逐题流式生成：改善真实做题时等待模型输出的体验。
3. 增量 `index.json`：让大笔记库扫描真正变快，而不是只记录元数据。
4. 完整 CI 冒烟：覆盖扫描、读笔记、mock LLM、保存错题和重启恢复。
5. 本地质量门禁：补 pre-commit、formatter 和最小 lint。

---

## 路线图完成进度

### 第 1 周：可靠性地基 ✅（全部完成）

目标：保证数据不丢、错误不静默。

任务：

- ✅ 修正 settings store 的 LLM 连接测试
- ✅ `storage.rs` 增加原子写入
- ✅ 增加 `.bak` 恢复
- ✅ Web follow-up 找不到 session 时返回 error
- ✅ Web server 默认绑定 `127.0.0.1`
- ✅ 增加最小 Rust 测试

### 第 2 周：LLM 稳定性 ✅（全部完成）

目标：减少模型输出导致的失败。

任务：

- ✅ 引入结构化 schema
- ✅ 增加 JSON 解析校验
- ✅ 解析失败时自动修复一次
- ✅ 增加 raw LLM debug 日志
- ✅ 增加错误分类（结构化返回）
- ✅ 前端显示生成阶段

### 第 3 周：长期使用体验 ✅

目标：让它像真正日用工具。

任务：

- ✅ 保存 rootPath
- ✅ 保存展开目录
- ✅ 保存阅读位置
- ✅ 错题本分页
- ✅ 错题搜索
- ✅ 错题导出
- ✅ 设置页增加数据备份/恢复
- ✅ 设置页增加打开数据目录

### 第 4 周：规模化和发布 ✅（发布基础和轻量索引第一阶段已完成）

目标：让大笔记库和正式发布更稳。

任务：

- ✅ 目录扫描忽略规则
- ✅ session 定期清理
- ✅ README 增加备份与故障恢复说明
- ✅ 冒烟测试脚本
- ✅ 轻量索引 `index.json`

---

## 当前完成度总结

| 优先级 | 内容 | 进度 | 已完成 | 未完成 |
|--------|------|------|--------|--------|
| P1 数据可靠性 | JSON 原子写入、数据管理 | ✅ ~95% | 原子写入、去重、备份恢复、文件状态、打开数据目录、导出错题 | 错题本中期（jsonl） |
| P2 运行风险 | LLM 连接、session 持久化、Web 安全 | ✅ 100% | 全部完成 | — |
| P3 LLM 稳定性 | JSON 校验、流式展示、能力探测 | ◐ ~95% | schema 校验、多题型支持、答案归一/高亮一致性（含选项文本、本地化分隔符、英文 `and` 连接与单选文本内 `and` / 逗号边界）、raw 日志、阶段进度、自动修复、能力探测 | 真正逐题 NDJSON 流式生成 |
| P4 笔记体验 | 目录扫描、阅读器、错题本 | ◐ ~95% | 滚动/图片/本地绝对图片路径/AVIF 附件/降级、分页、模式过滤、导出、搜索、大纲代码块边界与重复标题 id、frontmatter 标题解析与正文剥离、复习、盲区标签过滤、Markdown 扩展名兼容、扫描取消、工作区切换清理、打开/恢复失败不污染选中路径、阅读器读取竞态保护、错题筛选竞态保护、保存筛选一致性、轻量 index.json 第一阶段 | index.json 尚未用于增量扫描 |
| P5 测试门禁 | Rust/前端测试、冒烟流程 | ◐ ~90% | 60 Rust + 152 前端测试、冒烟脚本、CI/CD、README 备份文档、发布检查清单 | CI 冒烟偏浅；无本地 pre-commit / formatter / lint |

### 最推荐下一步

按投入产出比排序：

1. ~~导出错题~~ ✅ 已完成
2. ~~前端生成阶段进度~~ ✅ 已完成
3. ~~打开数据目录按钮~~ ✅ 已完成
4. ~~session 定期清理~~ ✅ 已完成
5. ~~LLM raw debug 日志~~ ✅ 已完成
6. ~~错题搜索~~ ✅ 已完成
7. ~~笔记内搜索~~ ✅ 已完成
8. ~~大纲目录~~ ✅ 已完成
9. ~~解析失败自动修复~~ ✅ 已完成
10. ~~README 备份与故障恢复说明~~ ✅ 已完成
11. ~~冒烟测试脚本~~ ✅ 已完成
12. ~~LLM 能力探测~~ ✅ 已完成
13. ~~错题复习流程~~ ✅ 已完成
14. ~~CI/CD~~ ✅ 已完成
15. ~~发布检查清单~~ ✅ 已完成

后续可选方向：
- `mistakes.jsonl` 迁移
- 真正逐题 NDJSON 流式生成
- `index.json` 增量复用
- 完整 CI 冒烟流程
- pre-commit / formatter / lint
