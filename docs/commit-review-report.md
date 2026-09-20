# Commit Review Report — KnowteQuiz

> 审查日期：2026-09-20 | 审查范围：`origin/main..main`（40 个提交，31 个非合并提交）

---

## 1. 审查结论

**全部 31 个非合并提交审查通过。** 每个提交的实际代码变更均与其提交信息描述的功能一致，未发现多余或无关代码。

| 类别 | 数量 |
|------|------|
| 功能（feat） | 7 |
| 修复（fix） | 13 |
| 重构（refactor） | 3 |
| 文档（docs） | 2 |
| 测试（test） | 1 |
| 杂项（chore/ci） | 2 |
| **总计** | **31** |

---

## 2. 逐提交审查结果

### 功能类（feat）

| 提交 | 信息 | 文件数 | +/− | 判定 | 备注 |
|------|------|--------|-----|------|------|
| `4a45076` | 自定义主题系统（背景色、强调色、背景图、毛玻璃） | 14 | +776/−171 | ✅ MATCH | 新增 BackgroundLayer、AppearanceSettings、useTheme；SettingsModal 拆为双标签页 |
| `8af81fa` | MP4 动态背景（播放/暂停/停止截图/恢复） | 11 | +509/−12 | ✅ MATCH | 新增 VideoBackground 组件，BackgroundLayer 集成视频，设置面板增加视频控制 |
| `62c6d2f` | 真·毛玻璃面板（可调模糊半径） | 15 | +126/−9 | ✅ MATCH | GlassmorphismConfig 增加 blur 字段，clampBlur 工具，CSS backdrop-filter，TitleBar/StatusBar 打标 |
| `d42725a` | 测验结果页新增"重做错题"按钮 | 7 | +35 | ✅ MATCH | QuizResult 新增 retryWrong 事件，quiz store 增加 retryWrongAnswers() |
| `1759e5c` | 每日活跃打卡追踪 | 11 | +73/−8 | ✅ MATCH | WorkspaceState 增加 streak_days/last_active_date，StatusBar 显示 🔥N 天 |
| `e90c9d3` | 诊断报告标注来源笔记 | 6 | +15/−1 | ✅ MATCH | DiagnosisReport 新增 noteTitle/notePath 属性，QuizSession 传入 |
| `065a7d8` | SM-2 间隔重复调度 | 21 | +400/−49 | ✅ MATCH | Rust/TS 双端实现 SM-2 算法，MistakeEntry 新增 ease_factor/interval_days/next_review_date，ErrorBook 增加 due 筛选，MistakeDetail 改为 4 按钮评分 |

### 修复类（fix）

| 提交 | 信息 | 文件数 | +/− | 判定 |
|------|------|--------|-----|------|
| `b16d624` | 安全加固、代码去重、内存泄漏防护 | 12 | +251/−151 | ✅ MATCH |
| `c34b777` | Web 端文件读取绑定笔记根目录 + 拒绝非 Markdown | 2 | +171/−8 | ✅ MATCH |
| `9f00a1c` | mistakes.jsonl 写入加锁 + 重 I/O 移入 spawn_blocking | 2 | +60/−4 | ✅ MATCH |
| `ee1b5f8` | 毛玻璃下保留自定义背景色 + CSS 变量重置 + 背景图去重 | 4 | +65/−45 | ✅ MATCH |
| `31a7fb3` | 视频 stop/screenshot/resume 接线 + 媒体大小限制 | 9 | +237/−27 | ✅ MATCH |
| `38bec7e` | 资源管理器/面板容器不再遮挡毛玻璃 | 3 | +12/−2 | ✅ MATCH |
| `8e4517f` | 自定义壁纸可读性遮罩 | 9 | +56 | ✅ MATCH |
| `ea50cd1` | 尊重 prefers-reduced-motion | 2 | +23 | ✅ MATCH |
| `824620b` | 视频恢复播放等待 loadedmetadata | 1 | +33/−10 | ✅ MATCH |
| `366d43f` | 弃用 settings.theme 死字段 | 4 | +6/−3 | ✅ MATCH |
| `e4fe2a7` | 毛玻璃面板加 CSS containment | 1 | +3 | ✅ MATCH |
| `55f794d` | 颜色选择器 label-input 关联 | 1 | +4/−2 | ✅ MATCH |
| `38bec7e` | 修复侧栏容器遮挡毛玻璃 | 3 | +12/−2 | ✅ MATCH |

### 修复类（fix）

| 提交 | 信息 | 文件数 | +/− | 判定 |
|------|------|--------|-----|------|
| `b16d624` | 安全加固、代码去重、内存泄漏防护 | 12 | +251/−151 | ✅ MATCH |
| `9f00a1c` | 错题 JSONL 写入加锁 + 重 I/O 移入 spawn_blocking | 2 | +60/−4 | ✅ MATCH |
| `c34b777` | Web 端文件读取绑定笔记根目录 | 2 | +171/−8 | ✅ MATCH |
| `ee1b5f8` | 毛玻璃保留自定义背景色 + CSS 变量重置 + 背景图统一 | 4 | +65/−45 | ✅ MATCH |
| `ea50cd1` | 尊重 prefers-reduced-motion | 2 | +23 | ✅ MATCH |
| `824620b` | 视频 seek 竞态修复 | 1 | +33/−10 | ✅ MATCH |
| `366d43f` | 弃用 settings.theme | 4 | +6/−3 | ✅ MATCH |
| `8e4517f` | 可读性遮罩滑杆 | 9 | +56 | ✅ MATCH |
| `616fb57` | clippy dead_code 修复 | 1 | +1 | ✅ MATCH |
| `865fdcd` | web_server::start 错误处理 | 2 | +17/−11 | ✅ MATCH |
| `8764354` | llm_service 返回 Result | 3 | +40/−50 | ✅ MATCH |
| `0726bf7` | 韩文错字 + i18n + 去重复别名 + 类型安全 | 9 | +16/−9 | ✅ MATCH |

### 重构类（refactor）

| 提交 | 信息 | 文件数 | +/− | 判定 |
|------|------|--------|-----|------|
| `2a9f76c` | 引入 AppError 枚举 | 12 | +352/−219 | ✅ MATCH |
| `afbe904` | fs_service 异步化（tokio::fs） | 5 | +95/−74 | ✅ MATCH |
| `967cc16` | 提取共享会话管理 + 模板缓存 + useRequestId | 6 | +133/−118 | ✅ MATCH |

### 文档 / 测试 / CI

| 提交 | 信息 | 文件数 | +/− | 判定 |
|------|------|--------|-----|------|
| `4d4a9a9` | 补充测试（+15 前端，+9 Rust） | 7 | +351/−3 | ✅ MATCH |
| `a596ea1` | 文档更新（测试数 + 新功能说明） | 2 | +45/−4 | ✅ MATCH |
| `dec4d3c` | 文档同步（修正测试数 + 删除已完成遗留项） | 2 | +1/−2 | ✅ MATCH |
| `888e5c7` | CI 冒烟测试修复（预构建二进制） | 1 | +6/−2 | ✅ MATCH |

---

## 3. 跨提交观察

以下不是"错误"，但值得注意的模式性现象：

### 3.1 `ui_language` 默认值被改了两次

| 提交 | 变更 |
|------|------|
| `b16d624` | `defaults.ts` 从 `'zh-CN'` → `'en'` |
| `467dc05` | 同上（i18n 迁移时再次设置） |

**结论**：不是 bug——第一次是意外副产物，第二次是刻意设定。最终值 `'en'` 正确，但提交历史有冗余。

### 3.2 SM-2 数据模型变更导致 5 个测试文件重写

`065a7d8`（SM-2）修改了 `MistakeEntry` 数据模型，导致 `QuizResult.vue`、`QuizSession.vue`、`mistake.test.ts`、`mistakes.test.ts`、`MistakeDetail.test.ts` 5 个文件必须同步更新 `MistakeEntry` 字面量。这是正常的级联影响，但说明在添加模型字段时应更早同步测试数据。

### 3.3 31 个本地分支未清理

当前 `git branch` 列出 31 个本地分支，全部已合并到 main。建议合并后删除：

```bash
git branch --merged main | grep -v '^\*' | xargs git branch -d
```

### 3.4 `main` 领先 `origin/main` 40 个提交，从未推送

所有变更仅存在于本地，无远程备份。

---

## 4. 统计摘要

| 指标 | 数值 |
|------|------|
| 非合并提交 | 31 |
| 审查通过（MATCH） | 31 / 31（100%） |
| 总插入行 | ~4,268 |
| 总删除行 | ~1,218 |
| 新增文件 | ~15（含 test、composable、utility） |
| 修改文件 | ~85 |
| Rust 测试 | 83 passed |
| 前端测试 | 189 passed（29 files） |
| 类型检查 | clean |
| 构建 | 4.71s |

---

## 5. 建议优化项

以下不构成"修复"，但值得关注：

| # | 类别 | 建议 | 优先级 |
|---|------|------|--------|
| 1 | 工程 | 推送到 `origin/main`——40 个提交零远程备份 | P0 |
| 2 | 工程 | 清理 31 个已合并本地分支 | P0 |
| 3 | 数据 | `mistakes.jsonl` 无 `ease_factor`/`interval_days` 的旧条目需兼容——当前用 `#[serde(default)]` 填默认值，但首次加载时可能需要批量初始化 | P2 |
| 4 | 工程 | `Cargo.toml` 中 `tauri-build` 同时出现在 `[build-dependencies]` 和 `[dependencies]`，重复声明 | P2 |
| 5 | 工程 | 依赖版本落后（axum 0.7、tower-http 0.5、dirs 5、pinia 2、vite 6） | P3 |
| 6 | UX | 毛玻璃 opacity 滑杆范围 0–100，但 CSS 用 `color-mix` 时 100% 等于完全不透明——考虑上限设为 50 | P3 |
| 7 | 文档 | `AGENTS.md` 中 `__TAURI_INTERNALS__` 的类型声明未记录 | P3 |
| 8 | 测试 | SM-2 Rust `sm2_update` 与 TS `sm2Update` 算法一致已验证，但缺少跨语言一致性测试（同一输入在两种实现中产出相同结果） | P3 |

---

## 6. 总结

**代码质量良好。** 31 个提交全部通过审查——代码变更与提交信息一一对应，未发现多余代码或功能偏离。主要的新功能（主题系统、视频背景、毛玻璃、SM-2）均有对应测试覆盖，文档也已同步更新。

当前最紧迫的行动是**推送到远程仓库**（40 个提交零备份）和**清理已合并分支**（31 个本地分支）。其余优化项可按优先级逐步处理。
