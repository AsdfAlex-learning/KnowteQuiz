# KnowteQuiz 项目复盘与优化建议（2026-10）

> 复盘日期：2026-10-04 | 方法：全量 git 历史（201 commits）+ 前端/后端/质量设施三路代码审查 + 文档与实际核对
>
> 结论先行：**整体健康度 B+**。地基（数据可靠性、安全默认值、测试文化、双运行时抽象）扎实；但深查发现 3 个已验证的 P0 正确性缺陷（本次已修复），以及一批文档未记录的结构性债务。测试全绿 ≠ 功能都对——Web 诊断路由失效、`isTauri` 未调用这类问题都缺"真实路由/真实环境"的端到端覆盖。
>
> **进展**：批次 1（P0 + 数据安全 + 卫生项）、批次 2（P1 健壮性 7/8）、批次 3（P2 架构整理）与批次 4 核心（工程设施）均已完成，见 §6–§8；#7 已随批次 3 修复。方向确认与后续路线图见 §5。

---

## 1. 演进复盘

201 个提交（2026-06-01 → 2026-09-20），约 13,300 行前端（TS/Vue）+ 5,800 行 Rust，共 30 个前端测试文件 / 191 用例 + 91 个 Rust 测试。四个阶段：

| 阶段 | 主题 | 评价 |
|---|---|---|
| 6 月 | 核心功能（出题/诊断/错题本）+ 数据可靠性 | 迭代快，fix 密集但方向正确 |
| 7 月 | 规模化（index.json、增量扫描）+ CI | 提前还了性能债 |
| 8 月 | i18n（4 语言）、JSONL 迁移、CI 冒烟、数据管理 | 工程化成熟期 |
| 9 月 | 外观系统（主题/视频/毛玻璃）、SM-2、streak、安全加固、AppError 重构 | 新功能引入了新债务（见 §4） |

### 做得好的（本次核实属实）

- 数据可靠性：原子写入 + `.bak` 恢复 + JSONL 迁移 + 进程级写锁 + 备份/恢复/数据状态。
- 安全默认值：127.0.0.1 绑定、CORS 白名单、web 端 canonicalize 路径绑定、asset 类型白名单。
- 测试文化：组件挂载级测试 + router 级 web_server 测试；几乎零 `console.*` / TODO / `@ts-ignore`。
- 竞态防护：4 个异步 store 中 3 个有请求令牌；错题写入锁有并发回归测试。
- 文档习惯：optimization-plan 持续追踪、commit-review；此前 commit-review 的两个 P0（40 提交未推送、31 分支未清理）均已完成。

### 文档"已完成"与实际的偏差

| 文档说法 | 实际 |
|---|---|
| "连接失败文案 i18n ✅" | SettingsModal 又出现硬编码 "Connection failed" 等 10+ 处（i18n 回归） |
| "笔记内搜索 ✅" | `window.find()`（非标准 API）+ 计数固定为 `'?'`，计数 UI 无意义 |
| "MarkdownContent 双 MarkdownIt 实例是有意的" | 该组件零引用，是死代码 |
| 测试数（4 处文档 3 种数字：152/180/189） | 实际 30 文件 / 191 用例（本次已同步修正） |

---

## 2. P0 缺陷（均已验证，本次已修复 ✅）

| # | 问题 | 位置 | 修复 |
|---|---|---|---|
| 1 | **Web 模式诊断追问/报告路由失效**：路由用 `{session_id}` 占位符，但 axum 0.7（matchit 0.7）只认 `:param`，大括号是字面量——两条路由永远匹配不上，请求落入 SPA fallback（POST → 405）。前端 `services/quiz.ts:110,127` 调的正是这两个 URL，桌面模式走 Tauri command 不受影响，CI 冒烟不覆盖 diagnose 流程，因此测试全绿 | `web_server.rs` 路由注册 | 改为 `:session_id`；路由表提取为 `build_router()`（单一事实来源）；新增 follow_up + report 两条 router 级回归测试（旧代码下它们分别得到 405/200-HTML，必然失败） |
| 2 | **UTF-8 字节切片 panic**：`&raw[..raw.len().min(200)]` 在第 200 字节切断多字节字符（中文输出常态）会 panic，且恰好发生在"LLM 输出解析失败"这条异常处理路径上 | `quiz_engine.rs`（parse_quiz_response / parse_diagnosis_initial） | 新增 `raw_preview()` 按 char boundary 截断；新增 3 个测试（含 201 字节中文边界用例） |
| 3 | **Web 模式手动输入笔记路径永不显示**：模板 `v-if="!isTauri && ..."` 引用导入的函数本身（恒为真值），应为 `!isTauri()` | `FolderSelector.vue:3` | 修正调用；新增组件测试（web 模式显示 / 桌面隐藏） |

## 3. 数据安全快修（本次已完成 ✅）

- **`write_json_path` 加进程级锁**：`.tmp` 路径固定，并发写同一文件会交错损坏 tmp→rename 序列（mistakes.jsonl 有锁，通用 JSON 写入没有）。已加 `JSON_WRITE_LOCK` + 8 线程并发回归测试。
- **统一 Web 与桌面数据目录**：web 端原用 `%APPDATA%\knowtequiz`，桌面用 `%APPDATA%\com.knowtequiz.app\knowtequiz`，`--mode=both` 时数据静默分裂。web 端已改用与 Tauri `app_data_dir()` 一致的路径（`storage::default_app_data_dir()`，镜像 `<data_dir>/<identifier>`），并带一次性旧目录迁移（copy-if-missing，幂等，含测试）。

## 4. 结构性问题清单（待处理）

### P1 — 数据与健壮性

> 2026-10-04 批次 2 更新：除 #7（涉及 test-connection 接口设计，归入批次 3 的设置 UI 合并）外全部修复。

| # | 问题 | 位置/证据 | 状态 |
|---|---|---|---|
| 4 | 最大 50MB 的视频以 **base64 存进 settings.json**（~67MB 字符串），且每次答题完成（streak）、切语言都触发整文件重写 | `AppearanceSettings.vue`（readAsDataURL）、`types/settings.ts` | ✅ 已修（9c33610）：媒体存 `<data_dir>/media/`，settings 存路径 + data-URL 自动迁移 + CSP 补 `media-src` |
| 5 | settings 整文件读-改-写在 4 个 store 并发执行，last-write-wins 互相覆盖（如布局快照覆盖新写入的滚动位置） | reader/explorer/layout/settings 的 persist 路径 | ✅ 已修（90ecf69）：`queueUpdate` 串行写队列，六处调用点迁移 |
| 6 | quiz 流式生成**无取消、无竞态令牌**：双击生成/中途切笔记会把两条流的事件混进一个 questions 数组；webStream 不接受 AbortSignal | `stores/quiz.ts`、`services/tauri.ts` | ✅ 已修（e3c7204）：request id + AbortSignal + Rust 侧断开停拉 LLM + dispatcher 去重 |
| 7 | SettingsModal 测连接时临时改 `settingsStore.settings.llm` 再还原，并发持久化会把**未提交的 LLM 配置写盘**；死掉的 SettingsPanel 版本语义相反（测的是已保存配置） | `SettingsModal.vue` | ❌ 未修（批次 3）：需 test-connection 接受待测 LlmConfig 参数，随设置 UI 合并一起做 |
| 8 | webStream SSE 解析：多行 `data:` 只取第一行（丢载荷）、单条 JSON 解析失败抛出整条流 | `services/tauri.ts` | ✅ 已修（bcc0f67）：按 SSE 规范聚合多行 data，坏事件跳过不中断 |
| 9 | LLM 客户端 60s **总超时**会截断长流式输出；客户端断开后 spawn 的任务继续烧完整个生成 | `llm_service.rs`、quiz_engine 流循环 | ✅ 已修（7be89cd + e3c7204）：connect 10s / 流式无总超时 + 90s chunk 空闲判死；接收端断开即停 |
| 10 | web 端 `/api/quiz/generate`、`/api/quiz/diagnose` 的路径未走 `resolve_note_path` 绑定——任意本地文件可被注入 prompt | `quiz_engine.rs`、web diagnose handler | ✅ 已修（24beac5）：`fs_service::resolve_note_path` 双端统一绑定 |
| 11 | `useTheme` 深度 watcher 包含 `video_time`，视频播放期间**每秒重应用整套主题 CSS 变量** | `useTheme.ts`、`BackgroundLayer.vue` | ✅ 已修（09eb149）：watch 源改为 applyTheme 实际读取的字段 |

### P2 — 代码卫生与一致性

> 2026-10-05 批次 3 更新：除「SM-2 跨语言一致性」（已用共享黄金向量测试缓解）、「AppError 类型化序列化」（顺延）外，本节全部处理完毕。

- ~~**死代码群约 810 行**~~ ✅ 已删除（SettingsPanel 370 行 + common 六组件 357 行 + utils/markdown.ts 未用导出）。
- ~~SettingsPanel 与 SettingsModal 双 UI~~ ✅ 已合并：Panel 优势特性（备份/恢复结果、修改时间、busy 标签、能力标签）全部进入 Modal，并补 dialog a11y（role/Esc/焦点陷阱）与 7 个组件测试。
- ~~Tauri 侧阻塞 IO~~ ✅ 已统一 spawn_blocking（backup/status/restore/cleanup/pick_folder）。
- ~~`response_format` URL 启发式~~ ✅ 已持久化 probe 结果到 settings.json，`call_llm` 优先采用（`wants_json_mode`）。
- ~~SM-2 无直接单测~~ ✅ 已补共享黄金向量测试（Rust 侧；与 TS 同向量）。
- ~~debug 日志无上限/同秒覆盖~~ ✅ uuid 后缀 + 100 条保留上限；session 清理启动时自动执行。
- ~~web 端 mark_mistake_reviewed 500~~ ✅ 类型化请求体（400），冒烟 payload 同步修复（顺带发现并修复 CI/冒烟用错 `quiz.language` 键的问题）。
- ~~笔记内搜索 window.find~~ ✅ 重写为 DOM mark 高亮 + 真实计数。
- ~~题目/解析纯文本渲染~~ ✅ 题干/选项/解析/诊断报告全部走 Markdown+KaTeX（html:false 防 XSS）。
- 无障碍剩余项（树节点键盘导航、resize 手柄）未处理，顺延。
- AppError 序列化为纯字符串（前端无法按类别处理错误）——顺延，需单独一轮设计前后端错误契约。

### P3 — 工程设施

> 2026-10-05 批次 4 更新：#12–#14、#17、#18 已完成；#15（发布自动化）与 #16（依赖大版本升级）按用户决定顺延下一轮。

| # | 问题 | 状态 |
|---|---|---|
| 12 | **无 JS linter**（仅 Prettier） | ✅ ESLint 9 flat config（vue3 + ts recommended）+ lint-staged + CI 步骤；存量死变量已修 |
| 13 | vitest 无配置文件 | ✅ `vitest.config.ts`（jsdom 默认）+ `@vitest/coverage-v8` + `test:coverage`；13 处散落注释清除 |
| 14 | CI 缺失项 | ✅ timeout-minutes ×4、concurrency、permissions、Windows job（advisory 起步）、smoke cargo 缓存、`cargo fmt --check` |
| 15 | 发布自动化为零 | ❌ 顺延（tauri-action + tag + 版本单源，需独立验证环境） |
| 16 | 依赖老化 | ❌ 顺延（axum 0.8 时 `:param` 需换回 `{param}`；pinia 3、vite 7） |
| 17 | 提交规范未强制 | ✅ commitlint + husky commit-msg hook（Conventional Commits） |
| 18 | 杂项 | ✅ `.gitignore` 补 `.codegraph/`、`.agents/`、`.zcode/`、`coverage/`；`npm test` 别名；冒烟脚本 `--data-dir` 封闭化（不再污染真实数据） |

---

## 5. 路线图（v2，2026-10-04 用户方向确认）

> 定位确认：**长期自用工具，足够好用后在 GitHub 开源分享**。节奏：小步快跑，每批端到端验证后再定下一批。功能构想（计划模式 + 知识图谱）已纳入路线图。

- **批次 1（已完成）**：P0 ×3 + 数据安全两项 + 卫生项（依赖/`--passWithNoTests`/文档测试数与 repo URL）。
- **批次 2（P1 健壮性，已完成）**：媒体移出 settings.json、settings 写入队列、quiz 流竞态与取消、SSE 加固、LLM 超时拆分、路径绑定、主题 watcher 拆分、i18n 回归清理。#7 归入批次 3。
- **批次 3（P2 架构整理）**：删除死代码群（~810 行）；SettingsModal/SettingsPanel 合一（抽 composable），顺带修 #7（test-connection 接受待测 LlmConfig）；题目/解析走 Markdown+KaTeX 渲染；Tauri 侧阻塞 IO 统一 spawn_blocking；a11y 基础（对话框焦点陷阱、树键盘导航）。
- **批次 4（P3 工程设施，直接服务开源）**：ESLint（或 oxlint）+ eslint-plugin-vue；vitest 统一配置 + 覆盖率；CI 加固（Windows job / timeout-minutes / concurrency / permissions / cargo 缓存 / cargo fmt --check）；tauri-action 发布自动化 + tag + 版本号单源；commitlint + dependabot；依赖升级（axum 0.8 时 `:param` 需换回 `{param}`——0.8 语法反转；pinia 3、vite 7）。
- **批次 5（计划模式 + 知识图谱 MVP）**：LearningPlan 数据模型（学习范围=工作区/目录/笔记清单、预计时长、目标掌握度映射 SM-2 目标）；知识脉络管道 = 逐文档 LLM 抽取 outline（按 size+time 增量缓存，复用 index.json 思路）→ 程序按目录结构/frontmatter 链接自动拼接图谱 → LLM 仅做最终一致性校验；出题按图谱节点与计划范围选题。OCR 暂不做。届时需设计评审：图谱存储格式（graph.json 邻接表 vs 节点表）、UI 入口（新 Panel tab）、与每日复习流的关系。
- **批次 6（学习统计面板）**：正确率趋势、复习完成率、计划进度、按笔记/标签薄弱点分布（数据源 mistakes.jsonl + streak + 批次 5 的 plan）。
- **批次 7（知识库增强）**：全库搜索（index.json 扩展标题+正文索引）、多笔记联合出题、wikilink/嵌入块兼容。

## 6. 批次 1 修改清单

| 文件 | 变更 |
|---|---|
| `src-tauri/src/web_server.rs` | 路由 `{session_id}` → `:session_id`；提取 `build_router()`；数据目录统一 + 旧目录迁移；+2 router 回归测试 |
| `src-tauri/src/services/quiz_engine.rs` | `raw_preview()` char-boundary 截断（2 处调用点）；`InitialDiagnosis` 补 `#[derive(Debug)]`；+3 测试 |
| `src-tauri/src/services/storage.rs` | `JSON_WRITE_LOCK` 写入锁；`default_app_data_dir()` / `migrate_legacy_web_data()`；+3 测试 |
| `src/components/Explorer/FolderSelector.vue` | `!isTauri` → `!isTauri()` |
| `src/components/Explorer/FolderSelector.test.ts` | 新增（2 用例） |
| `src-tauri/Cargo.toml` | 移除重复 `tauri-build` 与未使用的 `thiserror`；`tower` 移入 dev-dependencies |
| `package.json` | 移除 `--passWithNoTests` |
| `README.md` / `README.zh-CN.md` / `CONTRIBUTING.md` / `AGENTS.md` | 测试数同步（91/191/30 文件）、repo URL 修正、services 清单补全 |

验证：`cargo test` 91 passed / `cargo clippy -D warnings` clean / `vue-tsc --noEmit` clean / `vitest run` 30 files 191 passed。

## 7. 批次 2 修改清单（P1 健壮性加固，2026-10-04）

| 提交 | 内容 |
|---|---|
| `7be89cd` | LLM 超时拆分：connect 10s；流式请求换用无总超时的专用 client + 90s chunk 空闲判死；非流式保持 60s |
| `24beac5` | 路径绑定：`resolve_note_path` 下沉 `fs_service`，generate/diagnose 初轮/report/Tauri submit 全部绑定笔记根目录；+2 越界测试 |
| `bcc0f67` | webStream SSE 加固：多行 data 聚合、坏事件跳过不中断（测试契约有意变更） |
| `e3c7204` | quiz 流竞态与取消：store request id + AbortSignal；Tauri 转发循环断链 break；后端流循环 `tx.is_closed()` 停拉 LLM；dispatcher 去重 |
| `90ecf69` | settings 串行写队列 `queueUpdate`，六处调用点迁移；失败向上抛错由调用方处理 |
| `09eb149` | 主题 watcher 拆分：只 watch applyTheme 消耗的字段，video_time 不再触发重放 |
| `b2c2a7d` | i18n 回归清理：SettingsModal 8 处 + QuizResult/TitleBar/MistakeDetail/note.ts；4 语言新增 titlebar/quiz.saved/mistakes.ref/enter_path_prompt 键；locale 键一致性测试 |
| `9c33610` | 媒体移出 settings.json：`media_service`（uuid 命名、扩展白名单、大小上限）、Tauri `save_media_file`（raw IPC）、`POST/GET /api/data/media`、前端 `saveMediaFile(s)`、data-URL 自动迁移、CSP `media-src`；+7 测试 |

验证（批次 2 完成时）：`cargo test` 100 passed / clippy `-D warnings` clean / `vue-tsc` clean / `vitest run` 32 files 201 passed。

## 8. 批次 3 + 批次 4 修改清单（2026-10-05）

**批次 3（P2 架构整理）**

| 提交 | 内容 |
|---|---|
| `751b4d4` | #7 修复：test-connection 接受待测 LlmConfig（Tauri 命令/Axum 可选 body/前端 service），不再触碰已保存配置 |
| `6d94a2b` | SettingsModal 合并 Panel 全部优势特性 + dialog a11y（role/aria-modal/Esc/初始焦点/焦点陷阱） |
| `df597a9` | 删除生产死代码 SettingsPanel（370 行）及其测试 |
| `a602312` | 题目/选项/解析/诊断报告接入 `renderQuizMarkdown`（html:false + KaTeX）；删除 markdown.ts 五个未用导出（含 extractText no-op bug） |
| `8db6590` | 删除 6 个零引用 common 组件（357 行） |
| `25460fe` | 笔记内搜索重写：DOM mark 高亮 + 真实计数 N/M + 恢复原 HTML；移除 `window.find` 与唯一一处 `as any` |

**批次 4 核心（工程设施）**

| 提交 | 内容 |
|---|---|
| `ae0fabe` | Tauri 命令阻塞 IO 统一 spawn_blocking（backup/status/restore/cleanup/pick_folder）；删除 storage 三个无调用方包装 |
| `0e1da20` | mark_mistake_reviewed 类型化请求体（400）；修复 CI/冒烟脚本错误 payload（`id` → `mistake_id`+`quality`）与错误设置键（`quiz.language` → `default_language`） |
| `f3461d5` | probe 结果持久化到 settings.json；`wants_json_mode` 替代 URL 启发式 + 单测 |
| `3123ce9` | SM-2 四分支 + EF 下限黄金向量测试（与 TS 同向量） |
| `e767c84` | debug 日志 uuid 后缀 + 100 条保留上限；启动时自动清理过期 session（桌面 setup + web start） |
| `7730599` | `--data-dir` CLI 贯通；CI 冒烟与 smoke-test.ps1 改用临时目录（hermetic） |
| `d71ec75` | `cargo fmt` 一次性格式化（CI fmt 门禁前置） |
| `591cc47` | `vitest.config.ts`（jsdom 默认）+ coverage 工具与 include 配置；13 处环境注释清除；`npm test` / `test:coverage` 别名 |
| `11098cb` | ESLint flat config（vue3+ts recommended）+ lint-staged 集成；存量死变量/死代码修复（OptionCard base、QuizResult emit、DiagnosisChat props、useTheme 常量、start.cjs/setup.cjs） |
| `6ae86de` | CI：timeout ×4、concurrency、permissions、`cargo fmt --check`、smoke cargo 缓存、Windows cargo-test job（advisory） |
| `832b6e0` | commitlint（husky commit-msg）+ dependabot（npm/cargo/actions 每周）+ `.gitignore` 补工具目录 |

验证（批次 3+4 完成时）：`cargo test` 104 passed / `cargo clippy -D warnings` clean / `cargo fmt --check` clean / `eslint .` clean / `vue-tsc` clean / `vitest run` 32 files 207 passed / `npm run build` OK。
