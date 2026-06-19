# KnowteQuiz 全面优化方案

目标：不仅“可用”，而是能作为长期自用工具稳定运行，保证数据可靠、功能可恢复、体验顺滑、后续可维护。

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

## 第一优先级：数据可靠性

### 1. JSON 写入改为原子写入

当前 `settings.json` / `mistakes.json` 是直接覆盖写入。如果程序崩溃、断电或写入中断，文件可能损坏。

建议改为：

- 写入 `filename.tmp`
- flush / sync
- rename 覆盖正式文件
- 写入前保留 `filename.bak`
- 读取失败时自动尝试 `.bak`
- 给 `settings.json`、`mistakes.json` 增加 `version`

优先涉及文件：

- `src-tauri/src/services/storage.rs`
- `src-tauri/src/services/config.rs`

### 2. 错题本改为更适合长期积累的存储

当前保存错题是读取整个数组、插入、再整体写回。错题多了以后会变慢，也更容易损坏。

建议路线：

- 短期：继续使用 `mistakes.json`，但增加备份和分页加载
- 中期：改为 `mistakes.jsonl`，一行一条错题，追加写入
- 增加去重策略，避免同一题反复保存
- 增加导出功能

### 3. 增加数据管理能力

设置页建议加入：

- 打开数据目录
- 立即备份
- 恢复备份
- 导出错题
- 显示数据文件大小
- 显示最近写入时间

## 第二优先级：修正长期运行风险

### 1. LLM 连接测试统一走服务层

当前 `src/stores/settings.ts` 里存在直接从前端请求 LLM endpoint 的逻辑，这和项目约定冲突，也容易被 Tauri CSP 限制。

应统一使用：

- `src/services/settings.ts`
- Rust 后端的 `test_connection`
- Web 端 `/api/test-connection`

建议删除 store 内部直连 LLM 的实现。

### 2. 诊断 session 不应只存在内存

当前诊断会话存在内存 HashMap 中。程序关闭、刷新页面、服务重启后，诊断流程会丢失。

建议：

- 将未完成诊断 session 临时落盘到 `sessions/`
- 会话完成后写入错题完整上下文
- follow-up 找不到 session 时返回明确 error
- 定期清理过期 session

涉及文件：

- `src-tauri/src/commands/quiz.rs`
- `src-tauri/src/web_server.rs`
- `src-tauri/src/services/quiz_engine.rs`

### 3. Web server 默认只监听本机

当前 Web server 绑定 `0.0.0.0`，并使用 permissive CORS。长期运行时安全面偏大。

建议：

- 默认绑定 `127.0.0.1`
- 只有显式传入 `--host=0.0.0.0` 时才开放局域网
- CORS 默认只允许本机和 Vite dev origin

## 第三优先级：LLM 输出稳定性

### 1. 不要只靠“从文本里抠 JSON”

当前后端通过 `extract_json_block()` 从 LLM 输出中提取 JSON。长期使用本地模型时，容易遇到：

- JSON 半截
- 字段名偏差
- 输出解释文字
- 答案格式不符合预期

建议：

- 定义严格 Rust schema
- 解析失败时自动发起一次“JSON 修复请求”
- 保存 raw LLM response 到 debug 日志
- 校验字段完整性

校验规则示例：

- 题目不能为空
- 选择题必须有选项
- 单选答案必须落在选项内
- 多选答案必须能解析成多个选项
- 简答题必须有参考答案和解释

### 2. 真正流式展示生成过程

当前虽然请求 LLM stream，但后端是先积累完整内容，再统一解析并发送题目。

建议：

短期：

- 前端显示阶段进度：
  - 读取笔记
  - 请求模型
  - 生成中
  - 解析题目
  - 完成

中期：

- 要求模型输出 NDJSON
- 每生成一题就 emit 一个 chunk
- 前端逐题显示

### 3. LLM 能力探测

不同 OpenAI-compatible endpoint 支持能力不同。建议设置中加入：

- 是否支持 `response_format`
- 是否支持 stream
- 最大上下文长度
- 默认模型
- 连接测试结果详情

错误提示应区分：

- 网络不可达
- API key 错误
- 模型不存在
- 上下文过长
- JSON 解析失败
- 模型输出格式错误

## 第四优先级：笔记库长期使用体验

### 1. 目录扫描优化

当前每次全量递归扫描 Markdown。小库没问题，大库会卡。

建议：

- 忽略 `.git`、`node_modules`、`.obsidian/cache` 等目录
- 保存最近 rootPath
- 保存展开目录状态
- 保存当前选中文件
- 增加扫描取消机制
- 中期加入轻量 `index.json`

`index.json` 可记录：

- path
- mtime
- size
- title
- headings
- tags

### 2. 阅读器增强

长期阅读需要：

- 记住每篇笔记滚动位置
- 当前笔记搜索
- 大纲目录
- 图片相对路径解析
- Markdown 渲染失败时降级显示原文
- 支持跳转到错题相关笔记

### 3. 错题本增强

建议补齐：

- 搜索
- 按笔记过滤
- 按模式过滤：Basic / Advanced
- 按时间排序
- 按知识盲区标签过滤
- 标记已复习
- 复习次数
- 最近复习时间
- 导出 Markdown / JSON

## 第五优先级：测试和质量门禁

当前项目没有测试、lint、CI。长期自用至少需要基础验证。

### 1. Rust 单元测试

重点覆盖：

- JSON 原子写入
- `.bak` 恢复
- settings 默认值和迁移
- mistakes 保存和读取
- quiz JSON 解析
- diagnosis JSON 解析
- note scan 忽略规则

### 2. 前端服务测试

重点覆盖：

- `webStream()` SSE 解析
- services 的 Tauri / Web 分支
- quiz store 状态流转
- settings store 持久化流程

### 3. 最小冒烟流程

发版前至少验证：

1. 启动 Web mode
2. 加载设置
3. 扫描 fixture notes 目录
4. 读取一篇 Markdown
5. mock LLM 生成题目
6. 保存错题
7. 重启后错题仍存在

## 推荐实施路线图

### 第 1 周：可靠性地基

目标：保证数据不丢、错误不静默。

任务：

- 修正 settings store 的 LLM 连接测试
- `storage.rs` 增加原子写入
- 增加 `.bak` 恢复
- Web follow-up 找不到 session 时返回 error
- Web server 默认绑定 `127.0.0.1`
- 增加最小 Rust 测试

### 第 2 周：LLM 稳定性

目标：减少模型输出导致的失败。

任务：

- 引入结构化 schema
- 增加 JSON 解析校验
- 解析失败时自动修复一次
- 增加 raw LLM debug 日志
- 增加错误分类
- 前端显示生成阶段

### 第 3 周：长期使用体验

目标：让它像真正日用工具。

任务：

- 保存 rootPath
- 保存展开目录
- 保存阅读位置
- 错题本分页
- 错题搜索
- 错题导出
- 设置页增加数据备份/恢复

### 第 4 周：规模化和发布

目标：让大笔记库和正式发布更稳。

任务：

- 目录扫描忽略规则
- 轻量索引 `index.json`
- 增加发布前检查脚本
- Windows release 自测清单
- README 增加备份与故障恢复说明

## 最推荐先做的事情

第一件事建议做：

**数据可靠性包**

包括：

- `storage.rs` 原子写入
- `.bak` 自动恢复
- `settings.json` / `mistakes.json` version
- 错题保存测试
- 设置页增加“打开数据目录”和“立即备份”

原因：

这是长期自用工具的地基。只要真实学习数据会进入这个 app，第一优先级就不是功能更多，而是数据不能轻易损坏、丢失或无法恢复。
