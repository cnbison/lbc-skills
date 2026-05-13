# CLAUDE.md

This file is the **development guide** for Claude Code (claude.ai/code) when working **inside this repository**. It tells Claude what this repo is, where things live, and how to design / create / adapt skills here.

> 想了解仓库里**有哪些 skill、怎么使用**？请看 [README.md](./README.md)。本文件只关心**开发约束**，不当使用手册。

---

## 1. 仓库定位

`lbc-skills` 是一个**AI-native skill 工坊**：

- **核心目的**：设计、创建、改编、迭代 Claude Code skill（声明式的 prompt engineering artifact）。
- **核心受众**：skill 开发者本人 + Claude Code 在协作开发时的自我约束。
- **不是**：终端用户的使用手册，也不是 skill 安装器。
- **语言**：中文优先（Chinese-first），skill 描述与触发词同时提供中英双语。

**Claude 在本仓库的默认任务**：阅读现有 skill → 写新 skill / 改 skill → 暴露到 `.claude/skills/` → 提交并 push。

---

## 2. 仓库布局

仓库刻意把 **skill 源码** 和 **Claude Code 的加载路径** 分开：

| 路径 | 角色 |
|------|------|
| `skills/<name>/` | **真源**。所有 skill 文件（`SKILL.md` / `skill.md`、脚本、references、assets）都放这里。 |
| `.claude/skills/<name>` | **软链接**，指向 `../../skills/<name>`。Claude Code 在会话开始时只扫描这里。 |
| `docs/` | 跨 skill 的设计与对比文档（skill-loading-paths、skill-format-comparison 等）。 |
| `output/` | 各 skill 运行产物的统一落地目录（如 `output/daily-news/`、`output/follow-builders/`），不是源码，可被 `.gitignore`。 |

为什么要双路径？

- Claude Code 只从 `.claude/skills/`（项目级）或 `~/.claude/skills/`（用户级）加载 skill。放在 `skills/` 下的源文件对运行时**不可见**。
- 但把源码放在 `skills/` 下让仓库在 GitHub 上可浏览、便于跨项目复制、不用区分"install vs source"。软链接是桥。

详见 [`docs/skill-loading-paths.md`](docs/skill-loading-paths.md)。

---

## 3. Skill 的三类形态

| 形态 | 标志 | 代表 |
|------|------|------|
| **纯 prompt skill** | 仅 `SKILL.md` + 可选 `references/` | `dou-wentao-perspective`、`ljg-read`、`ljg-roundtable`、`mofa-concept`、`nuwa-skill`、`ljg-roundtable` |
| **Python 流水线 skill** | 带 `tools/`、`venv/`、`scripts/`、`requirements.txt` | `daily-news`、`follow-builders`、`agents-skill-podcastifier` |
| **MoFA 类 Rust 二进制 skill** | 带 `manifest.json`、`Cargo.toml`、`src/`、`architecture.dot` | `mofa-slides`、`mofa-fm`、`mofa-podcast-*`、`mofa-cli` |

三类的对比与权衡：见 [`docs/skill-format-comparison.md`](docs/skill-format-comparison.md)。

**选择原则**：能用 prompt 解决的不引入脚本，能用脚本解决的不引入 Rust。脚本类只做**确定性数据工作**（采集、清洗、TTS、ffmpeg 合并等），所有 LLM 生成由 Agent 在 SKILL.md 指导下完成，避免在脚本里硬编码 API key。

---

## 4. Skill 文件格式约束

所有 skill 的入口都是 `SKILL.md`（少数旧 skill 用 `skill.md`，新建一律 `SKILL.md`）。

### 4.1 YAML Frontmatter（必填）

```yaml
---
name: skill-name                 # kebab-case，唯一
description: >-                  # 必含触发词（中英双语）
  一句话定位 + Triggers: 触发词1, 触发词2, /command。
version: 0.1.0                   # 可选
author: name                     # 可选
always: false                    # 可选；true 表示常驻加载
requires_bins: python3, ffmpeg   # 可选；声明系统二进制依赖
requires_env: GEMINI_API_KEY     # 可选；声明环境变量依赖
---
```

`description` 是 Claude Code 自动激活 skill 的关键。**触发词必须显式列出**，否则 skill 永远不会被触发。

### 4.2 Body 结构

```markdown
## Usage

<example>
User: trigger phrase
Assistant: [Expected behavior]
</example>

## Instructions

1. 第一步...
2. 第二步...
```

### 4.3 命名与输出约定

- **skill 名**：kebab-case，与目录名一致。
- **文件输出**：生成的产物用时间戳前缀，便于排序与去重：`{YYYY-MM-DD}--{topic}__{type}.{ext}`，例如 `概念解剖-道__concept.md`。
- **输出落地路径**：
  - `~/Documents/notes/` — 文档类（markdown、org-mode）。
  - `~/Documents/scripts/` — 播客脚本类。
  - `./skill-output/<skill>-<timestamp>/` — 多文件流水线产物（slide、音频、视频段）。
  - `./output/<skill-name>/` — 各 skill 运行产物的统一落地目录（如 `output/daily-news`、`output/follow-builders`）。
- **Skill 解析与使用指南文档**：当用户对某个具体 skill 做深度解析、编写使用指南或技术分析时，生成的 markdown 文档默认保存到 `./docs/<skill-name>.md`（或 `./docs/<skill-name>-analysis.md`），文件名使用 skill 名 + 文档类型后缀，便于集中管理与检索。

---

## 5. MoFA 类 skill 的额外约束

MoFA（Model-on-File Architecture）skill 是 Rust 二进制 + 风格化 pipeline 的组合。新建 MoFA 类 skill 时必读：

### 5.1 目录骨架

```
skills/mofa-<name>/
├── SKILL.md              # skill 定义
├── manifest.json         # 二进制依赖与元数据
├── Cargo.toml            # Rust 依赖
├── src/
│   ├── main.rs           # CLI 入口
│   └── pipeline/         # pipeline 实现
├── styles/               # TOML 风格定义（slides 类）
├── scripts/              # 安装/工具脚本
└── architecture.dot      # pipeline 架构图（Graphviz）
```

### 5.2 关键模式

1. **二进制依赖**：在 `manifest.json` 里通过 `requires_bins` / `requires_bin` 声明，运行时检查。
2. **环境变量**：API key 在 `SKILL.md` frontmatter 用 `requires_env` 声明。
3. **输出路径**：相对路径 `skill-output/<skill>-<timestamp>/`，禁止硬编码绝对路径。
4. **架构图**：用 `architecture.dot` 描述 pipeline 数据流。

### 5.3 `mofa-cli` 是共享内核

`mofa-slides`、`mofa-podcast-*` 等共享同一个 `mofa` CLI 二进制（由 `mofa-cli` 提供）。新增 MoFA 子 skill 时，应优先复用 `mofa <subcommand>`，而不是新建独立二进制。

---

## 6. 创建新 Skill 的步骤（核心 SOP）

```bash
# 1) 在源目录建骨架
mkdir -p skills/<name>
$EDITOR skills/<name>/SKILL.md     # 写 YAML frontmatter + Usage + Instructions

# 2) 暴露到加载路径（关键，否则 Claude 看不到）
ln -s ../../skills/<name> .claude/skills/<name>

# 3) 加进 git
git add skills/<name> .claude/skills/<name>

# 4) 启动新会话验证 skill 是否被自动识别（看 system-reminder 中的 skill 列表）

# 5) 提交并推送
git commit -m "feat(<name>): add new skill"
git push origin main
```

**新 skill 的验收清单**：

- [ ] `description` 含至少 3 个触发词，中英双语
- [ ] 至少一个 `<example>` 块
- [ ] `## Instructions` 用编号步骤，避免模糊指令
- [ ] 软链接已建并 `git add`（软链接本身也要进 git）
- [ ] 新会话能在系统提示里看到该 skill
- [ ] 至少跑通一次端到端（生成产物或打印输出）
- [ ] 在 `README.md` 的 skill 目录中追加一行

---

## 7. 语言与本地化约定

- **首要语言**：简体中文。
- **触发词**：中英文都要列。例如：`Triggers: 圆桌, roundtable, debate`。
- **生成文件标题**：中文优先 + 英文 type 后缀，例如 `概念解剖-道__concept.md`。
- **代码注释**：英文（保持跨项目可读）；用户面向的 prompt 文案：中文。

---

## 8. Git / 版本控制工作流（硬约束）

**所有代码修改完成后必须推送到远程仓库。本地不留未推送提交。**

```bash
git status                    # 1) 检查变更
git add <files>               # 2) 精确添加（不用 git add .）
git commit -m "<scope>: 描述"  # 3) 中文或英文描述均可，scope 用 skill 名
git push origin main          # 4) 立即 push
```

**禁止**：
- 在本地保留未推送提交
- `git add .` / `git add -A`（容易卷入 `.env`、临时产物）
- 强推 main（`push --force` 到 main）
- 跳过 git hooks（`--no-verify`）

---

## 9. 工作时的默认动作

当用户在本仓库内提出 skill 相关请求时，Claude 默认执行顺序：

1. **先读 `skills/<name>/SKILL.md`**，了解 skill 现状（不靠记忆）。
2. **改动遵循"最小变更"原则**：只动用户要求的部分，不顺手重构。
3. **改完立即建/更新软链接**，再 `git add` 包含两个路径。
4. **任务完成立刻 `git commit && git push`**，不留尾巴。
5. **新建 skill 后在 `README.md` 的目录里补一行**，保持 README 与 `skills/` 同步。

当涉及具体代码编写（Python 脚本、Rust 二进制、JS 工具等）时，遵循 [`docs/claudemd_zh.md`](docs/claudemd_zh.md) 中的通用原则：**思考先于编码、简洁优先、精准修改、目标导向**。

---

## 10. 仅与开发相关的速查

- skill 形态对比：[`docs/skill-format-comparison.md`](docs/skill-format-comparison.md)
- skill 加载路径机制：[`docs/skill-loading-paths.md`](docs/skill-loading-paths.md)
- 单个 skill 的设计笔记：散落在 `docs/<skill-name>.md`
- 终端用户的使用说明：[`README.md`](README.md)
