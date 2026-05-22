# daily-news Skill 分析报告

> 基于 skill-creator 框架审查，Claude Code 执行环境约束，及仓库 CLAUDE.md 规范。
> 分析时间：2026-05-18
> 分析范围：SKILL.md + manifest.json + run_pipeline.py + tools/daily_news.py + 目录结构

---

## 一、执行摘要

`daily-news` 是一个 **Python 流水线 + Agent 驱动** 的混合形态 skill，整体架构清晰（Python 做确定性数据工作，Agent 做 LLM 生成），但存在 **3 个代码级 bug**、**manifest 与 SKILL.md 触发词不一致**、以及**输出命名不符合仓库约定**等问题。

| 严重程度 | 数量 | 类别 |
|---------|:---:|------|
| 🔴 严重 | 3 | 代码 bug（运行时错误 / 逻辑错误） |
| 🟡 高 | 4 | 触发/结构/一致性 |
| 🟢 中 | 5 | 规范/命名/冗余 |
| ⚪ 低 | 3 | 优化建议 |

---

## 二、代码级 Bug（🔴 严重）

### Bug 1：logger 在定义前使用（`run_pipeline.py:30`）

```python
# 第 30 行
logger.warning("MoFA FM publisher not available...")  # ← logger 尚未定义

# 第 44 行
logger = logging.getLogger(__name__)  # ← logger 在这里才定义
```

**后果**：导入 `run_pipeline.py` 时立即抛出 `NameError`，整个模块无法加载。

**修复**：将 logger 的定义移到文件顶部（`logging.getLogger` 可以在 `basicConfig` 之前安全调用）。

---

### Bug 2：使用了未导入的 `timedelta`（`run_pipeline.py:64`）

```python
from datetime import datetime  # ← 只导入了 datetime，没有 timedelta

# 第 64 行
ref_date = datetime.strptime(date, '%Y-%m-%d') + timedelta(...)  # ← NameError
```

**后果**：传入 `--date` 参数时 pipeline 直接崩溃。

**修复**：`from datetime import datetime, timedelta`

---

### Bug 3：Step 编号逻辑错误（`run_pipeline.py:216`）

```python
logger.info("\n[Step 7/5] Publishing to MoFA FM...")
```

Pipeline.run() 只有 3 个步骤（Collect/Filter/TTS），MoFA publish 是第 4 个可选步骤。`7/5` 的编号来源不明，可能是复制粘贴的遗留。

**修复**：改为 `[Step 4/4]` 或 `[Optional] Publishing to MoFA FM...`

---

## 三、高优先级问题（🟡）

### 问题 4：description 触发词与 manifest.json 不一致

| 来源 | 触发词数量 | 内容差异 |
|------|----------|---------|
| manifest.json | 18 个 | 含 "collect news", "filter news", "summarize news" 等分阶段触发词 |
| SKILL.md description | 9 个 | 缺少分阶段触发词，缺少 "collect" / "filter" / "summarize" 场景 |

**后果**：用户说 "collect news" 时 manifest 认为应该触发，但 SKILL.md 的 description 中没有覆盖，Claude 可能不激活 skill。

**修复**：统一触发词列表，将 manifest 中的 18 个全部纳入 SKILL.md description。

---

### 问题 5：description 混入了功能参数说明

```yaml
description: >-
  ...
  Supports --date, --skip-tts, --config and per-stage re-runs.
```

`--date`, `--skip-tts`, `--config` 是 CLI 功能参数，不是触发场景。根据 skill-creator 的指导，description 应该只回答"做什么"和"何时触发"。功能参数属于 `## Instructions` 或 `## Usage`。

**修复**：从 description 中删除功能参数说明，移到 Instructions 的命令速查部分。

---

### 问题 6：SKILL.md 过长（208 行）

skill-creator 建议 SKILL.md body 保持在 `<500 行`，理想 `<300 行`。当前 208 行虽然未超限，但加上 code block 和详细步骤，context 占用较大。

主要膨胀点：
- "Agent 驱动的 AI 生成流程" 部分（步骤 2-4）占 70+ 行
- 文章和播客脚本的结构要求非常详细

**建议**：将文章/播客的具体格式要求抽离到 `references/article-template.md` 和 `references/podcast-template.md`，SKILL.md 中只保留引用指引。

---

### 问题 7：播客脚本字数与时长严重不匹配

```
时长：3-5 分钟
字数：1200-1800 字
```

正常中文播客语速约 150-200 字/分钟：
- 3 分钟 ≈ 450-600 字
- 5 分钟 ≈ 750-1000 字

1200-1800 字对应 **8-12 分钟**，与时长要求矛盾。

**修复**：将字数调整为 600-1000 字，或延长时长要求为 8-10 分钟。

---

## 四、中优先级问题（🟢）

### 问题 8：缺少 `version` 字段

manifest.json 中有 `version: 1.5.0`，SKILL.md frontmatter 中没有。

---

### 问题 9：输出文件命名不符合仓库约定

CLAUDE.md §4.3 约定：`{YYYY-MM-DD}--{topic}__{type}.{ext}`

当前命名：
- `claw_daily_{date}.md` ❌
- `claw_podcast_{date}.txt` ❌
- `claw_daily_{date}.mp3` ❌

建议改为：
- `{date}--claw-daily__article.md`
- `{date}--claw-daily__podcast.txt`
- `{date}--claw-daily__audio.mp3`

---

### 问题 10：日志路径依赖 `cwd`

```python
log_path = Path.cwd() / 'output' / 'daily-news' / 'pipeline.log'
```

如果用户从不同目录运行命令，日志会分散到不同位置。

**建议**：将日志固定到 skill 目录或统一输出目录。

---

### 问题 11：`scripts/generate_article.py` 和 `generate_podcast.py` 目的不明确

SKILL.md 明确说"AI 生成由 Agent 完成"，但 scripts/ 下存在这两个文件。如果它们是数据辅助（格式化、结构化），文件名具有误导性；如果它们包含 LLM 调用逻辑，则与"不需要 AI API Key"的设计冲突。

**建议**：检查这两个文件的实际内容，明确其职责。如果是辅助格式化，更名为 `format_article.py` / `format_podcast.py`。

---

### 问题 12：`.env` 文件是否被 git 追踪

`.env` 文件存在于 skill 目录中。虽然 `.gitignore` 有 `.env` 规则，但需要确认该文件是否已被意外提交到 git 历史。

---

## 五、低优先级优化（⚪）

### 建议 13：`<example>` 块可以更具体

当前 example：
```
Assistant: [Runs full pipeline for today]
```

建议改为展示实际输出路径：
```
Assistant: 已执行完整流水线。
- 文章：./output/daily-news/output/2026-05-18--claw-daily__article.md
- 播客脚本：./output/daily-news/output/2026-05-18--claw-daily__podcast.txt
- 音频：./output/daily-news/output/2026-05-18--claw-daily__audio.mp3
```

---

### 建议 14：增加 per-stage re-run 的示例

SKILL.md description 提到 "per-stage re-runs" 支持，但 Usage 中没有展示如何单独运行某个阶段（除 collect/filter/tts 外）。可以增加：

```
User: 只重新生成今天的文章，不要重新采集
Assistant: [读取已过滤的 JSON，直接生成文章和播客脚本]
```

---

### 建议 15：触发词可增加口语化表达

如："今天有啥新闻", "给我讲讲今天的 tech 新闻", "播客做好了没"

---

## 六、修改优先级建议

| 顺序 | 项 | 预计工作量 |
|:---:|---|:---|
| 1 | 修复 3 个代码 bug | 5 分钟 |
| 2 | 统一触发词（manifest ↔ SKILL.md） | 10 分钟 |
| 3 | 调整播客字数/时长 | 2 分钟 |
| 4 | 清理 description 中功能参数 | 5 分钟 |
| 5 | 增加 version 字段 | 1 分钟 |
| 6 | 重命名输出文件（需同步修改代码中的文件名生成逻辑） | 20 分钟 |
| 7 | 将文章/播客模板抽离到 references/ | 30 分钟 |
| 8 | 优化 `<example>` 块 | 10 分钟 |

---

## 七、跨文件引用关系图

```
SKILL.md
├── 引用 tools/daily_news.py（CLI 入口）
├── 引用 run_pipeline.py（底层 pipeline）
├── 引用 scripts/collect_news.py（阶段 1）
├── 引用 scripts/filter_news.py（阶段 2）
├── 引用 scripts/tts_generate.py（阶段 5）
└── 阶段 2-4 由 Agent 直接完成（不调用脚本）

manifest.json
└── 触发词（18 个）→ 应与 SKILL.md description 对齐
```

---

## 八、总体评估

| 维度 | 评分 | 说明 |
|------|:---:|------|
| 架构设计 | ⭐⭐⭐⭐⭐ | Python + Agent 分离清晰，不需要 AI API Key 是亮点 |
| 代码质量 | ⭐⭐⭐ | 3 个明显 bug，日志处理不严谨 |
| SKILL.md 规范 | ⭐⭐⭐⭐ | 结构完整，但 description 混入非触发内容 |
| 触发准确度 | ⭐⭐⭐ | manifest 与 SKILL.md 不一致，描述不够 pushy |
| 输出规范 | ⭐⭐⭐ | 命名不符合仓库约定 |
| 可维护性 | ⭐⭐⭐⭐ | 分阶段命令清晰，per-stage re-run 支持好 |
