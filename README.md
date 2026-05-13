# lbc-skills

> 一个面向中文用户的 Claude Code Skill 集合。

`lbc-skills` 收录了一组**AI-native skill**：可被 Claude Code 自动识别、按触发词激活、声明式定义行为的能力包。覆盖**信息流、思考与知识、语音播客、多模态生成**四大场景，共 **21 个 skill**，其中 **8 个 active**、**13 个 WIP / 待激活**。

- 想了解**如何在自己的项目里加载这些 skill**：见下方 [快速开始](#快速开始)。
- 想**贡献新 skill 或修改现有 skill**：先读 [`CLAUDE.md`](./CLAUDE.md)（开发约束）。
- 单个 skill 的详细说明：见对应 `skills/<name>/SKILL.md`。

---

## 快速开始

### 加载到你的 Claude Code 项目

Claude Code 只从 `.claude/skills/` 或 `~/.claude/skills/` 加载 skill。把本仓库里的某个 skill 软链接到你的项目即可：

```bash
# 1. clone 本仓库到你想放的位置
git clone https://github.com/<you>/lbc-skills.git ~/projects/lbc-skills

# 2. 在你的目标项目里建软链接
cd /path/to/your-project
mkdir -p .claude/skills
ln -s ~/projects/lbc-skills/skills/daily-news .claude/skills/daily-news

# 3. 重新打开 Claude Code 会话，触发词即可激活
```

或者把整个仓库当成项目 root 直接打开 Claude Code，仓库根目录的 `.claude/skills/` 已经预先建好 6 个 active skill 的软链接。

### 触发示例

```
> 生成日报           → 触发 daily-news
> builder digest    → 触发 follow-builders
> 用窦文涛的视角看…  → 触发 dou-wentao-perspective
> 陪我读这篇         → 触发 ljg-read
> 蒸馏 <人物>        → 触发 nuwa-skill
> save this          → 触发 second-brain
```

---

## Skill 目录

图例：✅ active（仓库根 `.claude/skills/` 已暴露）｜🚧 WIP（仅源码存在，未暴露，需自行软链接）

### 📰 信息流 / 内容生产

| Skill | 状态 | 一句话 | 触发词 | 依赖 |
|------|:--:|------|------|------|
| [`daily-news`](skills/daily-news/) | ✅ | RSS 采集 → 过滤去重 → Agent 摘要/文章/播客脚本 → TTS 合成 MP3 | 生成日报 / claw日报 / 今天的新闻 / 播报 | `python3` |
| [`follow-builders`](skills/follow-builders/) | ✅ | 监控 X 上的 AI builder 与播客，做日 / 周 digest，支持中英双语 | AI digest / builders digest / builder 更新 / industry insights | 无（从中心 feed 拉取） |
| [`ljg-read`](skills/ljg-read/) | ✅ | 伴读 agent：翻译、结构标注、深度提问、跨学科洞察 | 伴读 / 陪我读 / 读这篇 / read with me | — |
| [`news-summary`](skills/news-summary/) | 🚧 | 抓取国际 RSS 做新闻播报，可生成语音摘要 | news / 新闻 / briefing | RSS、可选 TTS |

### 🧠 思考 / 知识 / 视角

| Skill | 状态 | 一句话 | 触发词 | 依赖 |
|------|:--:|------|------|------|
| [`dou-wentao-perspective`](skills/dou-wentao-perspective/) | ✅ | 窦文涛的 5 个心智模型 + 7 条决策启发式，做思维顾问 | 窦文涛视角 / 锵锵模式 / 圆桌派视角 | — |
| [`nuwa-skill`](skills/nuwa-skill/) | ✅ | 女娲造人：输入人名 / 主题 / 模糊需求，自动调研并蒸馏出人物 skill | 造 skill / 蒸馏 XX / 女娲 / 造人 | — |
| [`second-brain`](skills/second-brain/) | ✅ | 由 Ensue 驱动的个人知识库，捕捉与检索理解 | save this / remember / my notes on | `ENSUE_API_KEY` |
| [`mofa-concept`](skills/mofa-concept/) | 🚧 | 8 维概念解剖（历史 / 辩证 / 现象学 / 语言学 / 形式化 / 存在主义 / 美学 / 元哲学），输出 md 报告 | 解剖概念 / 概念解剖 / explain concept | — |
| [`ljg-roundtable`](skills/ljg-roundtable/) | 🚧 | 结构化多视角辩论框架，主持人邀请代表人物围绕议题展开 | 圆桌讨论 / 圆桌 / roundtable / 辩论 | — |

### 🎙 语音 / 播客 / TTS

| Skill | 状态 | 一句话 | 触发词 | 依赖 |
|------|:--:|------|------|------|
| [`mofa-fm`](skills/mofa-fm/) | 🚧 | TTS 与声音克隆，内置 9 个预设音色，支持自定义 | voice / TTS / 语音 / 播报 | `mofa-fm` |
| [`mofa-podcast-0429`](skills/mofa-podcast-0429/) | 🚧 | 多人对话播客（1-5 人），支持情绪标签与 BGM cue，新版本 | podcast / 播客 / 多人对话 / 锵锵三人行 | `mofa-podcast` |
| [`mofa-podcast-old`](skills/mofa-podcast-old/) | 🚧 | mofa-podcast 的旧版本，保留作对比基线 | 同上 | `mofa-podcast` |
| [`agents-skill-podcastifier`](skills/agents-skill-podcastifier/) | 🚧 | 把邮件 / newsletter 切块、TTS 合成、ffmpeg 拼接，做成短播客 | podcastify / 把这封邮件做成播客 | `python3` `ffmpeg` |
| [`mofa-fm-api`](skills/mofa-fm-api/) | 🚧 | MoFA FM 播客平台的 API 客户端（节目、单集、搜索、热搜） | mofa.fm / fm api / 热搜 | `python3` |

### 🎬 多模态 / 视频 / 幻灯片

| Skill | 状态 | 一句话 | 触发词 | 依赖 |
|------|:--:|------|------|------|
| [`mofa-slides`](skills/mofa-slides/) | 🚧 | AI 整页 Gemini 图幻灯片，17+ 风格，4 种生成模式 | mofa ppt / slides / 幻灯片 / 用 mofa 做 PPT | `mofa` `GEMINI_API_KEY` |
| [`md-to-video`](skills/md-to-video/) | 🚧 | Markdown → AI 幻灯片 → TTS 旁白 → ffmpeg → MP4 | md to video / 脚本做视频 / 旁白 PPT | `mofa-slides` `mofa-fm` `ffmpeg` `GEMINI_API_KEY` |
| [`ffmpeg-video-editor`](skills/ffmpeg-video-editor/) | 🚧 | 自然语言 → ffmpeg 命令（剪辑、转码、压缩、提取音频等） | ffmpeg / 视频编辑 / 转码 | `ffmpeg` |

### 🎵 音乐

| Skill | 状态 | 一句话 | 触发词 | 依赖 |
|------|:--:|------|------|------|
| [`ace-music`](skills/ace-music/) | 🚧 | 通过 ACE-Step 1.5 免费 API 生成音乐 / 歌曲 / 配乐 / 翻唱 | create music / generate song / 作曲 / 配乐 | `ACE_MUSIC_API_KEY` |

### 🛠 基础设施

| Skill | 状态 | 一句话 | 触发词 | 依赖 |
|------|:--:|------|------|------|
| [`mofa-cli`](skills/mofa-cli/) | 🚧 | 所有 MoFA skill 共享的 CLI 二进制（slides / cards / comic / infographic / video） | 非用户触发，被其他 skill 依赖 | Rust toolchain |
| [`skill-creator`](skills/skill-creator/) | ✅ | 创建、修改、评估和优化 skill 的完整工作流（含 eval 框架与 description 优化） | create skill / 新建 skill / 优化 skill / skill eval | `python3` |
| [`find-skills`](skills/find-skills/) | ✅ | 在开放 skill 生态中发现和安装 agent skill | find a skill / how do I do X / discover skills / install skill | `npx skills` |

---

## MoFA CLI 速查

MoFA 类 skill 共享同一个 `mofa` CLI 二进制（由 `mofa-cli` 提供）：

```bash
# 生成幻灯片
mofa slides --style nb-pro --out deck.pptx --slide-dir imgs/ input.json

# 生成卡片
mofa cards --style minimal --out cards.pptx input.json

# 生成信息图
mofa infographic --style data --out infographic.png input.json

# 生成视频
mofa video --style cinematic --out video.mp4 input.json
```

`mofa-slides` 支持 4 种生成模式：

| 模式 | 描述 | 适用 |
|------|------|------|
| Mode 1: Image-only | 文字烧进 AI 图 | 快、好看、不可改 |
| Mode 2: Manual text overlay | 干净背景 + 定位文本框 | 像素级控制 |
| Mode 3: Auto-layout (VQA) | AI 出带文图，VQA 提文，原图除字 | 全自动可改 |
| Mode 4: PDF-to-PPTX | 把已有图导成可编辑 PPTX | 导入 / 转换 |

---

## 外部依赖一览

| 工具 / 变量 | 用途 | 使用它的 skill |
|------------|------|----------------|
| `mofa` | MoFA CLI（slides / cards / comic / infographic / video） | mofa-slides、mofa-cli、md-to-video |
| `mofa-fm` | TTS 与声音克隆 | mofa-fm、md-to-video、mofa-podcast-* |
| `mofa-podcast` | 多人对话播客 | mofa-podcast-0429、mofa-podcast-old |
| `ffmpeg` | 音视频处理 | md-to-video、mofa-podcast-*、ffmpeg-video-editor、agents-skill-podcastifier |
| `soffice` | PPTX 转换（可选） | md-to-video、mofa-slides |
| `python3` | Python 流水线 | daily-news、agents-skill-podcastifier、mofa-fm-api |
| `GEMINI_API_KEY` | 图像生成 + VQA | mofa-slides、md-to-video |
| `DASHSCOPE_API_KEY` | 文字消除（qwen-image-edit） | mofa-slides（auto-layout 模式） |
| `ENSUE_API_KEY` | Ensue 知识库 | second-brain |

---

## 文档

- [`CLAUDE.md`](./CLAUDE.md) — 开发指南（仓库定位、目录约定、skill 形态、新增 skill 的 SOP、Git 工作流）
- [`docs/skill-loading-paths.md`](./docs/skill-loading-paths.md) — `skills/` 与 `.claude/skills/` 双路径机制详解
- [`docs/skill-format-comparison.md`](./docs/skill-format-comparison.md) — 三类 skill 形态的对比与权衡
- `docs/<skill-name>.md` — 部分 skill 的设计笔记

---

## 贡献新 Skill

详见 [`CLAUDE.md` §6 创建新 Skill 的步骤](./CLAUDE.md#6-创建新-skill-的步骤核心-sop)。

简版：

```bash
mkdir -p skills/<name>
$EDITOR skills/<name>/SKILL.md
ln -s ../../skills/<name> .claude/skills/<name>
git add skills/<name> .claude/skills/<name>
git commit -m "feat(<name>): add new skill"
git push origin main
```

然后在本 README 的目录表里追加一行。
