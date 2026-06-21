# KnowteQuiz 全面优化方案 — 进度追踪

> 最后同步：2026-06-21 | 基于 git commit 历史自动标记
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
- ❌ 增加导出功能

相关 commit：`feat: paginate mistake book filters`、`fix: dedupe and filter mistakes`、`feat: track mistake save state`

### 3. 增加数据管理能力 ◐

设置页建议加入：

- ❌ 打开数据目录
- ✅ 立即备份（`Backup Data Now` -> `settingsService.backupData()`）
- ✅ 恢复备份（`Restore Latest Backup` -> `settingsService.restoreLatestBackup()`）
- ❌ 导出错题
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
- ❌ 定期清理过期 session

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

### 1. 不要只靠"从文本里抠 JSON" ✅（基础）/ ◐（高级）

当前后端通过 `extract_json_block()` 从 LLM 输出中提取 JSON。长期使用本地模型时，容易遇到：

- ✅ JSON 半截 — 增加多重解析策略
- ✅ 字段名偏差 — 结构化 schema 校验
- ✅ 输出解释文字 — `parse fenced quiz json` 支持围栏代码块
- ✅ 答案格式不符合预期 — 多种 answer 格式兼容

建议：

- ✅ 定义严格 Rust schema（`models/` 下的 struct）
- ❌ 解析失败时自动发起一次"JSON 修复请求"
- ❌ 保存 raw LLM response 到 debug 日志
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

- ◐ 前端显示阶段进度 — 目前只显示 "Generating questions..." 和错误信息，未实现细粒度阶段指示（读取笔记→请求模型→生成中→解析题目→完成）
- ✅ 流式错误传递（`fix: stream web diagnosis startup errors`、`fix: flush trailing sse messages`）

中期：

- ❌ 要求模型输出 NDJSON
- ❌ 每生成一题就 emit 一个 chunk
- ❌ 前端逐题显示

### 3. LLM 能力探测 ❌

不同 OpenAI-compatible endpoint 支持能力不同。建议设置中加入：

- ❌ 是否支持 `response_format`
- ❌ 是否支持 stream
- ❌ 最大上下文长度
- ❌ 默认模型
- ✅ 连接测试结果详情（结构化返回，但非能力探测）

---

## 第四优先级：笔记库长期使用体验

### 1. 目录扫描优化 ◐

当前每次全量递归扫描 Markdown。小库没问题，大库会卡。

建议：

- ✅ 忽略 `.git`、`node_modules`、`.obsidian/cache` 等目录（`fix: skip heavy folders when scanning notes`）
- ✅ 保存最近 rootPath（`fix: persist manually opened note root`）
- ✅ 保存展开目录状态（`feat: persist workspace state in settings`）
- ✅ 保存当前选中文件
- ❌ 增加扫描取消机制
- ❌ 中期加入轻量 `index.json`

### 2. 阅读器增强 ◐

长期阅读需要：

- ✅ 记住每篇笔记滚动位置（`feat: persist reader scroll positions`）
- ✅ 恢复阅读滚动（`feat: restore reader scroll in app`）
- ❌ 当前笔记搜索
- ❌ 大纲目录
- ✅ 图片相对路径解析（`fix: resolve markdown image assets`）
- ✅ Markdown 渲染失败时降级显示原文（`fix: fall back on markdown render errors`）
- ✅ 支持跳转到错题相关笔记（ErrorBook -> MistakeDetail 有 "Open Note" 按钮）

相关 commit：`feat: persist reader scroll positions`、`feat: restore reader scroll in app`、`fix: resolve markdown image assets`、`fix: fall back on markdown render errors`、`fix: cache markdown renderers by options`

### 3. 错题本增强 ◐

建议补齐：

- ❌ 搜索（按问题文本搜索）
- ✅ 按笔记过滤（`MistakeFilter.note_path`）
- ✅ 按模式过滤：Basic / Advanced（`ErrorBook.vue` 中的 filter bar）
- ❌ 按时间排序
- ❌ 按知识盲区标签过滤
- ❌ 标记已复习
- ❌ 复习次数（`MistakeEntry.review_count` 字段已定义但 review flow 未实现）
- ❌ 最近复习时间
- ❌ 导出 Markdown / JSON

相关 commit：`feat: paginate mistake book filters`、`feat: pass mistake filters from frontend`、`fix: dedupe and filter mistakes`

---

## 第五优先级：测试和质量门禁 ✅（基础）/ ❌（完整）

当前项目没有测试、lint、CI。长期自用至少需要基础验证。

### 1. Rust 单元测试 ✅

已添加 48 个 `#[test]` 测试，重点覆盖：

- ✅ JSON 原子写入
- ✅ `.bak` 恢复
- ✅ settings 默认值和迁移
- ✅ mistakes 保存和读取
- ✅ quiz JSON 解析
- ✅ diagnosis JSON 解析
- ✅ note scan 忽略规则

### 2. 前端服务测试 ✅

已添加 19 个测试文件，共 117 个测试用例：

- ✅ `webStream()` SSE 解析（`tauri.test.ts`）
- ✅ services 的 Tauri / Web 分支（`settings.test.ts`、`quiz.test.ts`、`mistake.test.ts`...）
- ✅ quiz store 状态流转（`quiz.test.ts`）
- ✅ settings store 持久化流程

### 3. 最小冒烟流程 ❌

发版前至少验证：

❌ 1. 启动 Web mode
❌ 2. 加载设置
❌ 3. 扫描 fixture notes 目录
❌ 4. 读取一篇 Markdown
❌ 5. mock LLM 生成题目
❌ 6. 保存错题
❌ 7. 重启后错题仍存在

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

### 第 2 周：LLM 稳定性 ◐（部分完成）

目标：减少模型输出导致的失败。

任务：

- ✅ 引入结构化 schema
- ✅ 增加 JSON 解析校验
- ❌ 解析失败时自动修复一次
- ❌ 增加 raw LLM debug 日志
- ✅ 增加错误分类（结构化返回）
- ❌ 前端显示生成阶段（目前仅有简单文本）

### 第 3 周：长期使用体验 ◐（部分完成）

目标：让它像真正日用工具。

任务：

- ✅ 保存 rootPath
- ✅ 保存展开目录
- ✅ 保存阅读位置
- ✅ 错题本分页
- ❌ 错题搜索
- ❌ 错题导出
- ✅ 设置页增加数据备份/恢复

### 第 4 周：规模化和发布 ❌（基本未开始）

目标：让大笔记库和正式发布更稳。

任务：

- ✅ 目录扫描忽略规则
- ❌ 轻量索引 `index.json`
- ❌ 增加发布前检查脚本
- ❌ Windows release 自测清单
- ❌ README 增加备份与故障恢复说明

---

## 当前完成度总结

| 优先级 | 内容 | 进度 | 已完成 | 未完成 |
|--------|------|------|--------|--------|
| P1 数据可靠性 | JSON 原子写入、数据管理 | ◐ ~80% | 原子写入、去重、备份恢复、文件状态 | 打开数据目录、导出错题 |
| P2 运行风险 | LLM 连接、session 持久化、Web 安全 | ✅ ~95% | 全部核心功能完成 | session 定期清理 |
| P3 LLM 稳定性 | JSON 校验、流式展示、能力探测 | ◐ ~55% | schema 校验、多题型支持 | 自动修复、raw 日志、阶段进度、能力探测 |
| P4 笔记体验 | 目录扫描、阅读器、错题本 | ◐ ~55% | 滚动/图片/降级、分页、模式过滤 | 搜索、大纲、复习、导出、index.json |
| P5 测试门禁 | Rust/前端测试、冒烟流程 | ◐ ~65% | 48 Rust + 117 前端测试 | 冒烟流程、CI、发布检查 |

### 最推荐下一步

按投入产出比排序：

1. **导出错题** — 设置页已有备份框架，增加导出 JSON/Markdown 工作量不大
2. **前端生成阶段进度** — 用户体验提升明显，实现简单
3. **打开数据目录按钮** — 单行命令调用，对用户数据管理安全感提升大
4. **session 定期清理** — 避免长期运行后 sessions 目录膨胀
5. **LLM raw debug 日志** — 排查模型输出问题的关键基础设施
