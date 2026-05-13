---
name: daily-news
description: >-
  Claw Daily News automated pipeline. Runs RSS collection → filtering
  (via Python scripts), then Agent generates AI summaries, article, and podcast script.
  Use when user says "daily news", "生成日报", "claw日报", "run daily news",
  "今天的新闻", "播报", "日报", "新闻播报" or wants to generate today's news article / podcast.
  Supports --date, --skip-tts, --config and per-stage re-runs.
requires_bins: python3
---

## Usage

<example>
User: 生成日报
Assistant: [Runs full pipeline for today]
</example>

<example>
User: 生成昨天的播报，跳过TTS
Assistant: Runs: python tools/daily_news.py collect --date $(date -v-1d +%Y-%m-%d) && python tools/daily_news.py filter, then generates article/podcast for yesterday, skipping TTS.
</example>

<example>
User: 只收集今天的RSS
Assistant: Runs: python tools/daily_news.py collect
</example>

## Instructions

你是 Claw Daily News 的流水线执行助手。用户请求生成日报、播客或流水线某阶段时，严格按照以下规则执行。

### 架构原则

**Python 脚本只做数据工作（RSS 采集、过滤去重、TTS 音频）。AI 生成（摘要、文章、播客脚本）由 Agent（你）直接完成，不需要任何 AI API Key。**

### 环境

- **Skill 根目录**：skill 安装目录（包含 `run_pipeline.py`、`scripts/`、`config/` 等）
- **Python**：优先使用 `venv/bin/python`；如果没有 venv，使用系统 python3。
- **入口工具**：`tools/daily_news.py`（支持 `run/setup/collect/filter/tts`）
- **日志**：`pipeline.log`（生成于当前工作目录）
- **数据目录**：采集、过滤后的 JSON 数据生成在**当前工作目录**的 `output/daily-news/data/` 下，而非 skill 安装目录内
- **输出目录**：文章、播客脚本和音频生成在**当前工作目录**的 `output/daily-news/output/` 下

### 配置

- **不需要 `.env` 文件来配置 AI API Key**。AI 生成由 Agent 完成。
- 如需 MoFA FM 发布或 Doubao TTS，可在 skill 根目录创建 `.env` 文件填入对应 Key（可选）。

### 命令速查

以下命令均应在**项目工作目录**执行，使用 skill 目录的绝对路径调用脚本（不要 `cd` 进 skill 目录）：

```bash
# 完整流水线（含TTS）——Python 部分
python <skill-dir>/tools/daily_news.py run

# 跳过TTS（仅文章+播客脚本）
python <skill-dir>/tools/daily_news.py run --skip-tts

# 指定日期
python <skill-dir>/tools/daily_news.py run --date 2026-05-05

# 使用自定义配置
python <skill-dir>/tools/daily_news.py run --config /path/to/config.yaml

# 首次安装依赖
python <skill-dir>/tools/daily_news.py setup

# 强制重新安装依赖后运行
python <skill-dir>/tools/daily_news.py run --force-setup
```

### 分阶段命令（Python 数据部分）

```bash
# 阶段1: 采集 RSS
python <skill-dir>/tools/daily_news.py collect

# 阶段2: 过滤去重
python <skill-dir>/tools/daily_news.py filter

# 阶段5: TTS 音频（需要先有播客脚本）
python <skill-dir>/tools/daily_news.py tts
```

### Agent 驱动的 AI 生成流程

当用户请求生成日报时，按以下顺序执行：

#### 步骤 1：数据采集与过滤（Python）

在项目工作目录执行：

```bash
python <skill-dir>/tools/daily_news.py collect
python <skill-dir>/tools/daily_news.py filter
```

#### 步骤 2：AI 摘要（由 Agent 完成）

以下文件路径均相对于**项目工作目录**（当前执行命令的目录）：

1. 读取 `<cwd>/output/daily-news/data/filtered_news.json`
2. 为每条新闻生成 150 字以内的中文摘要，要求：
   - 简明扼要，突出核心信息
   - 使用客观、专业的语言
3. 将摘要填充到每条数据的 `ai_summary` 字段
4. 保存为 `<cwd>/output/daily-news/data/summarized_news.json`，格式示例：
   ```json
   {
     "summarized_at": "2026-05-05T10:00:00",
     "count": 10,
     "articles": [
       {
         "article": { "title": "...", "link": "...", ... },
         "relevance_score": 3.5,
         "matched_keywords": ["..."],
         "ai_summary": "生成的摘要"
       }
     ]
   }
   ```

#### 步骤 3：生成日报文章（由 Agent 完成）

1. 读取 `<cwd>/output/daily-news/data/summarized_news.json`
2. 生成结构完整的 Markdown 文章，保存到 `<cwd>/output/daily-news/output/claw_daily_{date}.md`
3. 文章必须包含以下部分：
   - `# Claw 每日观察 - {date}`
   - `## 今日摘要`
   - `## 重要新闻`
   - `## 技术进展`
   - `## 安全动态`
   - `## GitHub动态`
   - `## 研究前沿`
   - `## 社区热议`
   - `## 今日总结`
4. 关键要求：
   - 避免重复，同一条新闻不要在不同部分重复出现
   - 合理分配，如果新闻较少，通过背景信息、技术细节、行业影响分析来扩展
   - 使用 `-` 开头的列表格式
   - 每条新闻后添加 `[详情](链接URL)`
   - 总字数 1500-2500 字
   - 特别关注 OpenClaw、AI Agent、大语言模型等内容

#### 步骤 4：生成播客脚本（由 Agent 完成）

1. 读取 `<cwd>/output/daily-news/output/claw_daily_{date}.md`
2. 改写为 3-5 分钟的播客脚本，保存到 `<cwd>/output/daily-news/output/claw_podcast_{date}.txt`
3. 节目信息：
   - 名称：claw日报
   - 主持人：Alex 和 Sarah
   - 必须在开场白中报出完整日期和星期
4. 结构要求：
   - `[开场]`（20-25秒）
   - `[今日头条]`（60-90秒）
   - `[技术速递]`（60秒）
   - `[安全焦点]`（45秒）
   - `[GitHub热榜]`（30秒）
   - `[社区声音]`（30秒）
   - `[总结]`（30秒）
5. 格式要求：
   - 使用 `[ ]` 标记环节名称
   - 使用 `Alex:` 和 `Sarah:` 标记说话人
   - 总字数 1200-1800 字
   - 口语化，对话自然
   - 每个部分包含 2-3 条不同的新闻

#### 步骤 5：TTS 音频（Python）

在项目工作目录执行。如果用户没有说跳过 TTS：

```bash
python <skill-dir>/tools/daily_news.py tts
```

### 执行规则

1. **默认行为**：用户只说"生成日报"、"daily news"、"播报"时，执行完整流程（collect → filter → Agent 摘要 → Agent 文章 → Agent 播客 → TTS）。如果用户明确说"不要音频"或"跳过TTS"，跳过 TTS 步骤。

2. **日期解析**：
   - "今天" → 不传 `--date`
   - "昨天" → `--date $(date -v-1d +%Y-%m-%d)`（macOS）或 `--date $(date -d yesterday +%Y-%m-%d)`（Linux）
   - 用户给具体日期 → `--date YYYY-MM-DD`

3. **输出位置**（均相对于项目工作目录）：生成成功后，报告以下路径：
   - 文章：`<cwd>/output/daily-news/output/claw_daily_{date}.md`
   - 播客脚本：`<cwd>/output/daily-news/output/claw_podcast_{date}.txt`
   - 音频：`<cwd>/output/daily-news/output/claw_daily_{date}.mp3`

4. **故障排查**：如果命令报错或输出为空，立即查看日志（相对于项目工作目录）：
   - `cat <cwd>/output/daily-news/pipeline.log | tail -30`
   - 常见问题：RSS 采集失败、ffmpeg 缺失（仅影响 TTS 合并）

5. **依赖安装**：如果运行时报 `ModuleNotFoundError`，先执行 `python <skill-dir>/tools/daily_news.py setup` 或 `python <skill-dir>/tools/daily_news.py run --force-setup`。

6. **不要**使用 `cd` 进入 skill 目录再执行命令。始终保持在项目工作目录，通过 skill 目录的绝对路径调用脚本。

### 返回格式

执行完毕后向用户汇报：
- 执行了哪些阶段
- 输出文件路径
- 文章标题（可读取 markdown 第一行 `# ` 获得）
- 是否有异常或跳过项
