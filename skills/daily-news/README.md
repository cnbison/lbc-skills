# Daily News Skill

Claw Daily News 自动化流水线 Skill。采用 **Agent + Skill** 架构：

- **Python 脚本**：RSS 采集 → 过滤去重 → TTS 音频
- **Agent（Claude Code）**：AI 摘要 → 文章生成 → 播客脚本

不需要配置任何 AI API Key，所有 LLM 生成工作由 Agent 直接完成。

## 安装

将本目录复制到 Claude Code 的 skills 目录：

```bash
# 项目本地安装
mkdir -p /path/to/your/project/.claude/skills
cp -r daily-news /path/to/your/project/.claude/skills/

# 或全局安装
mkdir -p ~/.claude/skills
cp -r daily-news ~/.claude/skills/
```

安装依赖：

```bash
python tools/daily_news.py setup
```

## 配置

**不需要配置 AI API Key。**

如需 MoFA FM 发布或 Doubao TTS，可在 skill 根目录创建 `.env` 文件填入对应 Key（可选）：

```bash
cp .env.example .env
# 编辑 .env，填入可选服务 Key
```

## 使用

### 完整流水线

由 Claude Code Agent 执行：

```
User: 生成日报
Claude: collect → filter → 生成摘要 → 生成文章 → 生成播客 → TTS
```

### Python 数据阶段（可独立运行）

```bash
python tools/daily_news.py collect    # 采集 RSS
python tools/daily_news.py filter     # 过滤去重
python tools/daily_news.py tts        # 生成音频（需先有播客脚本）
```

### 指定日期

```bash
python tools/daily_news.py collect --date 2026-05-05
python tools/daily_news.py filter --date 2026-05-05
```

### 跳过 TTS

```bash
python tools/daily_news.py run --skip-tts
```

## 文件结构

```
daily-news/
├── manifest.json       # Skill 元数据与触发词
├── SKILL.md            # Claude 指令定义（Agent 驱动流程）
├── README.md           # 本文件
├── requirements.txt    # Python 依赖
├── setup.sh            # 环境初始化脚本
├── run_pipeline.py     # Python 流水线入口（collect/filter/tts）
├── .env.example        # 可选环境变量模板（MoFA/Doubao）
├── .gitignore
├── config/
│   └── sources.yaml    # RSS 源配置
├── scripts/            # 各阶段模块
│   ├── config.py
│   ├── models.py
│   ├── collect_news.py      # RSS 采集
│   ├── filter_news.py       # 过滤去重
│   ├── summarize_news.py    # 摘要数据辅助（AI 由 Agent 完成）
│   ├── generate_article.py  # 文章数据辅助（AI 由 Agent 完成）
│   ├── generate_podcast.py  # 播客数据辅助（AI 由 Agent 完成）
│   ├── tts_generate.py      # TTS 音频
│   ├── mofa_client.py       # MoFA FM API 客户端
│   └── mofa_publish.py      # MoFA FM 自动发布
└── tools/
    └── daily_news.py        # 统一 CLI 入口
```

## 输出文件

- `daily-news/output/claw_daily_{date}.md` — 日报文章（Agent 生成）
- `daily-news/output/claw_podcast_{date}.txt` — 播客脚本（Agent 生成）
- `daily-news/output/claw_daily_{date}.mp3` — TTS 音频（Python 生成）

运行时 `daily-news/data/` 和 `daily-news/output/` 目录会自动创建在当前工作目录下，不需要打包在 skill 中。

## 依赖

- Python 3.8+
- 可选：ffmpeg（用于多语音 TTS 合并）
